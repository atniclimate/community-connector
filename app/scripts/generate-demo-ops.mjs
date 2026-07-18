import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const here = path.dirname(fileURLToPath(import.meta.url));
const repo = path.resolve(here, "../..");
const fixtures = path.join(repo, "fixtures");

// Flags:
//   --public-layer  give entities of non-person kinds whose label-bearing
//                   attribute is template-public a "public" presence, and make
//                   edges between two such entities "public", so the baked
//                   anonymous snapshot scope (P2.3/P2.5,
//                   docs/design/snapshot-viewer-scope.md) projects a real
//                   graph. Persons and all attribute-instance visibilities are
//                   never touched. Kinds with no public label attr (all of
//                   fisheries-committee, by design) are unaffected, so that
//                   fixture's output is byte-identical with or without the
//                   flag.
//   --out-dir <dir> write op logs somewhere other than fixtures/groups
//                   (testing only).
const args = process.argv.slice(2);
const publicLayer = args.includes("--public-layer");
const outDirFlagIndex = args.indexOf("--out-dir");
const outDirOverride = outDirFlagIndex === -1 ? null : args[outDirFlagIndex + 1];
const knownArgs = new Set(["--public-layer", "--out-dir"]);
for (const [index, arg] of args.entries()) {
  if (index === outDirFlagIndex + 1 && outDirFlagIndex !== -1) {
    continue;
  }
  if (!knownArgs.has(arg)) {
    console.error(`Unknown argument: ${arg}\nUsage: node scripts/generate-demo-ops.mjs [--public-layer] [--out-dir <dir>]`);
    process.exit(1);
  }
}
if (outDirFlagIndex !== -1 && (outDirOverride === undefined || outDirOverride.startsWith("--"))) {
  console.error("--out-dir requires a directory argument");
  process.exit(1);
}
const outDir = outDirOverride == null ? path.join(fixtures, "groups") : path.resolve(outDirOverride);

// Mirrors the label candidate list in app/src/viz/projection.ts entityLabel:
// the public layer only publishes entities the anonymous viewer could actually
// read a label for.
const labelAttrIds = ["display_name", "title", "common_name", "scientific_name"];

function kindHasPublicLabel(kind) {
  if (kind.id === "person") {
    return false;
  }
  return (kind.attributes ?? []).some(
    (attr) => labelAttrIds.includes(attr.id) && attr.default_visibility === "public",
  );
}
const schemaVersion = "0.1.0";
const groupVisibility = "group";
const publicVisibility = "public";
const tierT0 = "T0";
const tierT1 = "T1";
const entityCountPerKind = 20;
const edgeCount = 300;
const governanceCount = 3;
const firstSort = 1;
const lcgA = 1664525;
const lcgC = 1013904223;
const lcgMod = 0x100000000;
const researchGroupId = uuid(16);
const fisheriesGroupId = uuid(17);
const researchPersonBase = 1000;
const fisheriesPersonBase = 2000;
const entityBase = 10000;
const edgeBase = 30000;
const membershipBase = 50000;
const storyBase = 70000;
const opBase = 90000;

const specs = [
  {
    template: "research-network",
    groupId: researchGroupId,
    personBase: researchPersonBase,
    seed: 41,
    storyTitle: "Synthetic Research Story",
    storyNarration: "Synthetic coordination step",
  },
  {
    template: "fisheries-committee",
    groupId: fisheriesGroupId,
    personBase: fisheriesPersonBase,
    seed: 83,
    storyTitle: "Synthetic Committee Story",
    storyNarration: "Synthetic committee step",
  },
];

const wordA = [
  "Alder",
  "Birch",
  "Cedar",
  "Drift",
  "Ember",
  "Fern",
  "Harbor",
  "Juniper",
  "Kestrel",
  "Lumen",
];
const wordB = [
  "Riverbend",
  "Meadow",
  "Northstar",
  "Willow",
  "Hearth",
  "Stone",
  "Maple",
  "Field",
  "Brook",
  "Grove",
];
const tags = ["coordination", "fieldwork", "planning", "stewardship", "mapping"];

function uuid(n) {
  const hex = BigInt(n).toString(16).padStart(32, "0");
  return `${hex.slice(0, 8)}-${hex.slice(8, 12)}-${hex.slice(12, 16)}-${hex.slice(16, 20)}-${hex.slice(20)}`;
}

function lcg(seed) {
  let state = seed >>> 0;
  return () => {
    state = (Math.imul(state, lcgA) + lcgC) >>> 0;
    return state / lcgMod;
  };
}

function pick(random, values) {
  return values[Math.floor(random() * values.length)] ?? values[0];
}

function provenance(person, at) {
  return {
    origin: "authored",
    recorded_by: { human: person },
    responsible_human: person,
    recorded_at: at,
    custody: [],
    schema_version: schemaVersion,
  };
}

function operation(spec, sort, actor, kind) {
  const opId = uuid(opBase + sort);
  return {
    op_id: opId,
    group_id: spec.groupId,
    actor: { human: actor },
    responsible_human: actor,
    recorded_at: sort,
    sort_key: {
      hlc: { wall_ms: sort, counter: 0 },
      actor_key: `human:${actor}`,
      op_id: opId,
    },
    template_version: schemaVersion,
    kind,
    schema_version: schemaVersion,
  };
}

function groupCreate(spec, template, actor, sort) {
  return operation(spec, sort, actor, {
    op: "GroupCreate",
    group: {
      id: spec.groupId,
      name: template.name,
      template_id: template.template_id,
      template_version: schemaVersion,
      provenance: provenance(actor, sort),
      tier: tierT1,
      schema_version: schemaVersion,
    },
    template_json: JSON.stringify(template),
  });
}

function membershipOp(spec, person, role, sort) {
  return operation(spec, sort, person, {
    op: "MembershipAdd",
    membership: {
      id: uuid(membershipBase + sort),
      group_id: spec.groupId,
      person,
      role,
      lifecycle: "active",
      provenance: provenance(person, sort),
      schema_version: schemaVersion,
    },
  });
}

function attribute(attr, value, actor, sort) {
  return {
    value,
    visibility: attr.default_visibility ?? groupVisibility,
    tier_override: null,
    provenance: provenance(actor, sort),
  };
}

function attrValue(attr, label, index, random) {
  switch (attr.type) {
    case "text":
      return { type: "text", value: textValue(attr.id, label, index) };
    case "number":
      return { type: "number", value: index + 1 };
    case "enum":
      return { type: "enum", value: attr.values[index % attr.values.length] };
    case "tags":
      return { type: "tags", value: [pick(random, tags), pick(random, tags)] };
    case "date":
      return { type: "date", value: `2026-${String((index % 9) + 1).padStart(2, "0")}-15` };
    case "geo":
      return { type: "geo", value: { point: { lat: 40 + random() * 5, lon: -125 + random() * 5 } } };
    case "link":
      return linkValue(attr, label, index);
    case "media":
      return { type: "media", value: `synthetic-media-${index}` };
    default:
      return { type: "text", value: label };
  }
}

function linkValue(attr, label, index) {
  if (attr.format === "email") {
    return {
      type: "link",
      value: {
        url: `mailto:${label.toLowerCase().replaceAll(" ", ".")}.${index}@example.test`,
        format: "email",
      },
    };
  }
  return { type: "link", value: { url: `https://example.test/${index}`, format: "url" } };
}

function textValue(attrId, label, index) {
  if (attrId === "orcid") {
    return `https://example.test/orcid/${index}`;
  }
  return `${label} ${index}`;
}

function entityOp(spec, templateKind, entityId, owner, label, index, sort, random) {
  const attributes = {};
  for (const attr of templateKind.attributes ?? []) {
    attributes[attr.id] = attribute(attr, attrValue(attr, label, index, random), owner, sort);
  }
  return operation(spec, sort, owner, {
    op: "EntityCreate",
    entity: {
      id: entityId,
      group_id: spec.groupId,
      kind: templateKind.id,
      attributes,
      owner: templateKind.id === "person" ? owner : null,
      presence_visibility:
        publicLayer && kindHasPublicLabel(templateKind) ? publicVisibility : groupVisibility,
      lifecycle: "active",
      provenance: provenance(owner, sort),
      tier: tierT0,
      schema_version: schemaVersion,
    },
  });
}

function edgeOp(spec, edgeKind, from, to, edgeIndex, sort, actor, publicIds) {
  const publicEdge = publicLayer && publicIds.has(from) && publicIds.has(to);
  return operation(spec, sort, actor, {
    op: "EdgeCreate",
    edge: {
      id: uuid(edgeBase + edgeIndex),
      group_id: spec.groupId,
      kind: edgeKind.id,
      from,
      to,
      directed: edgeKind.directed,
      weight: edgeKind.weighted === "optional" ? 1 + (edgeIndex % 5) : null,
      attributes: {},
      visibility: publicEdge ? publicVisibility : groupVisibility,
      lifecycle: "active",
      provenance: provenance(actor, sort),
      tier: tierT0,
      schema_version: schemaVersion,
    },
  });
}

function storyOp(spec, entityIds, sort, actor) {
  return operation(spec, sort, actor, {
    op: "StoryCreate",
    story: {
      id: uuid(storyBase + sort),
      group_id: spec.groupId,
      title: spec.storyTitle,
      steps: entityIds.slice(0, 4).map((entity, index) => ({
        entity,
        narration: `${spec.storyNarration} ${index + 1}`,
      })),
      visibility: groupVisibility,
      lifecycle: "active",
      provenance: provenance(actor, sort),
      tier: tierT0,
      schema_version: schemaVersion,
    },
  });
}

function buildEntities(spec, template, random, startSort) {
  const ops = [];
  const byKind = new Map();
  const publicIds = new Set();
  let sort = startSort;
  let entityIndex = 0;
  for (const kind of template.kinds) {
    const ids = [];
    const publicKind = publicLayer && kindHasPublicLabel(kind);
    for (let index = 0; index < entityCountPerKind; index += 1) {
      const id = kind.id === "person"
        ? uuid(spec.personBase + index + 1)
        : uuid(entityBase + spec.personBase + entityIndex);
      const owner = uuid(spec.personBase + (index % entityCountPerKind) + 1);
      const label = `${pick(random, wordA)} ${pick(random, wordB)}`;
      sort += 1;
      ops.push(entityOp(spec, kind, id, owner, label, entityIndex + 1, sort, random));
      ids.push(id);
      if (publicKind) {
        publicIds.add(id);
      }
      entityIndex += 1;
    }
    byKind.set(kind.id, ids);
  }
  return { ops, byKind, publicIds, sort };
}

function compatibleIds(edgeKind, byKind, side, random) {
  const kinds = edgeKind[side];
  const kind = kinds[Math.floor(random() * kinds.length)] ?? kinds[0];
  return byKind.get(kind) ?? [];
}

function buildEdges(spec, template, byKind, publicIds, random, startSort) {
  const ops = [];
  let sort = startSort;
  for (let index = 0; index < edgeCount; index += 1) {
    const edgeKind = template.edge_kinds[index % template.edge_kinds.length];
    const fromIds = compatibleIds(edgeKind, byKind, "from", random);
    const toIds = compatibleIds(edgeKind, byKind, "to", random);
    const from = fromIds[Math.floor(random() * fromIds.length)] ?? fromIds[0];
    let to = toIds[Math.floor(random() * toIds.length)] ?? toIds[0];
    if (from === to && toIds.length > 1) {
      to = toIds[(toIds.indexOf(to) + 1) % toIds.length];
    }
    sort += 1;
    ops.push(edgeOp(spec, edgeKind, from, to, spec.personBase + index, sort, uuid(spec.personBase + 1), publicIds));
  }
  return { ops, sort };
}

function buildOps(spec) {
  const template = JSON.parse(readFileSync(path.join(fixtures, "templates", `${spec.template}.template.json`), "utf8"));
  const random = lcg(spec.seed);
  const actor = uuid(spec.personBase + 1);
  let sort = firstSort;
  const ops = [groupCreate(spec, template, actor, sort)];
  for (let index = 0; index < governanceCount; index += 1) {
    sort += 1;
    ops.push(membershipOp(spec, uuid(spec.personBase + index + 1), "governance", sort));
  }
  const entities = buildEntities(spec, template, random, sort);
  ops.push(...entities.ops);
  const edges = buildEdges(spec, template, entities.byKind, entities.publicIds, random, entities.sort);
  ops.push(...edges.ops);
  ops.push(storyOp(spec, [...entities.byKind.values()].flat(), edges.sort + 1, actor));
  return ops.map((op) => JSON.stringify(op)).join("\n") + "\n";
}

mkdirSync(outDir, { recursive: true });
for (const spec of specs) {
  writeFileSync(path.join(outDir, `${spec.template}.ops.jsonl`), buildOps(spec));
}
