// Generates fixtures/groups/atni-convention.ops.jsonl: the synthetic ATNI
// convention pilot fixture (S-R2, D-094c). Deliberately uneven so the reveal
// has visible structure (discovery-2026-09-12.md Track B):
//   - 12 people with exactly one committee tie ("edge of the network")
//   - 12 people on both Energy and Climate Resilience plus 1-2 more
//     committees (the heavy-overlap pair those two committees share)
//   - 36 people on 3-4 committees each, spread across all 15 so no
//     committee is isolated
// All entries are tier T1 (D-034). Every name/email/tribe/organization is
// fictional; every contact value uses the @example.test namespace (I1).
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const here = path.dirname(fileURLToPath(import.meta.url));
const repo = path.resolve(here, "../..");
const fixturesDir = path.join(repo, "fixtures");
const outDir = path.join(fixturesDir, "groups");

const schemaVersion = "0.1.1";
const tierT1 = "T1";
const groupVisibility = "group";

const PEOPLE_COUNT = 60;
const SINGLE_TIE_COUNT = 12;
const OVERLAP_COUNT = 12;
const COMMITTEE_COUNT = 15;
const ORG_COUNT = 12;
const ENERGY = 0;
const CLIMATE_RESILIENCE = 14;

function uuid(n) {
  const hex = BigInt(n).toString(16).padStart(32, "0");
  return `${hex.slice(0, 8)}-${hex.slice(8, 12)}-${hex.slice(12, 16)}-${hex.slice(16, 20)}-${hex.slice(20)}`;
}

const groupId = uuid(900000);
const personId = (i) => uuid(910000 + i + 1);
const committeeId = (c) => uuid(920000 + c + 1);
const orgId = (o) => uuid(930000 + o + 1);
const facilitator = personId(0);

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

function operation(sort, actor, kind) {
  const opId = uuid(980000 + sort);
  return {
    op_id: opId,
    group_id: groupId,
    actor: { human: actor },
    responsible_human: actor,
    recorded_at: sort,
    sort_key: { hlc: { wall_ms: sort, counter: 0 }, actor_key: `human:${actor}`, op_id: opId },
    template_version: schemaVersion,
    kind,
    schema_version: schemaVersion,
  };
}

function attribute(value, visibility, actor, sort) {
  return { value, visibility, tier_override: null, provenance: provenance(actor, sort) };
}

// --- Word banks (all fictional; none is a real Tribal Nation, org, or person) ---
const wordA = ["Alder", "Birch", "Cedar", "Drift", "Ember", "Fern", "Harbor", "Juniper", "Kestrel", "Lumen"];
const wordB = ["Riverbend", "Meadow", "Northstar", "Willow", "Hearth", "Stone", "Maple", "Field", "Brook", "Grove"];
const FICTIONAL_TRIBES = [
  "Cedar Hollow Nation", "Blue Ridge Band", "Silver Creek Tribe", "Stonewater Nation",
  "Fernwood Band", "Highridge Tribe", "Willowbend Nation", "Amber Valley Band",
  "Driftwood Tribe", "Copper Basin Nation", "Northgate Band", "Emberlake Nation",
];
const ROLE_TITLES = [
  "Committee Coordinator", "Program Director", "Youth Council Liaison", "Grants Manager",
  "Communications Lead", "Community Organizer", "Policy Analyst", "Field Coordinator",
  "Outreach Specialist", "Administrative Assistant",
];
const EVENT_TAGS = ["convention-2026", "monthly-meeting", "training-session", "policy-summit"];
const ORG_NAMES = [
  "Cedar Alliance Program", "Harbor Youth Coalition", "Kestrel Policy Institute", "Lumen Community Fund",
  "Northstar Tribal Enterprises", "Willow Grove Nonprofit", "Fieldstone Agency", "Brookside Partners",
  "Maple Ridge Foundation", "Ember Works Cooperative", "Driftline Solutions", "Hearthstone Council",
];
const ORG_TYPES = ["university", "agency", "tribal-nation", "ngo", "industry"];

// Committee names verbatim, in the ruled order (D-094c).
const COMMITTEES = [
  "Energy", "Taxation", "Education (K-12)", "ICWA", "Law & Justice", "Philanthropy",
  "Telecomms & Tech", "Food Sovereignty", "Economic Development", "Native Vote", "TERO",
  "Gaming", "Drug Abuse & Prevention", "Housing", "Climate Resilience",
];
// One thematic tag per committee, used to give person interest/specialty tags
// real correlation with their committee ties (visible Jaccard affinity, Track B).
const COMMITTEE_TAG = [
  "renewable-energy", "tax-policy", "stem-education", "child-welfare", "tribal-court",
  "grant-writing", "broadband-access", "food-sovereignty", "small-business",
  "voter-registration", "workforce-development", "gaming-regulation",
  "substance-prevention", "affordable-housing", "climate-adaptation",
];

// --- Structural assignment: which committees/orgs each person belongs to ---
const committeesFor = [];
for (let i = 0; i < SINGLE_TIE_COUNT; i += 1) {
  committeesFor.push([i % COMMITTEE_COUNT]);
}
for (let i = 0; i < OVERLAP_COUNT; i += 1) {
  const extra = i < 6 ? [(i + 1) % COMMITTEE_COUNT] : [(i + 1) % COMMITTEE_COUNT, (i + 5) % COMMITTEE_COUNT];
  committeesFor.push([ENERGY, CLIMATE_RESILIENCE, ...extra]);
}
const BULK_START = SINGLE_TIE_COUNT + OVERLAP_COUNT;
for (let i = BULK_START; i < PEOPLE_COUNT; i += 1) {
  const base = i % COMMITTEE_COUNT;
  const count = i % 2 === 0 ? 3 : 4;
  const steps = [0, 5, 10, 3];
  const set = new Set();
  for (let k = 0; k < count; k += 1) {
    set.add((base + steps[k]) % COMMITTEE_COUNT);
  }
  committeesFor.push([...set]);
}

const orgsFor = [];
for (let i = 0; i < PEOPLE_COUNT; i += 1) {
  const orgs = [i % ORG_COUNT];
  if (i % 10 === 0) {
    orgs.push((Math.floor(i / 10) + 6) % ORG_COUNT);
  }
  orgsFor.push(orgs);
}

// --- People ---
function displayName(i) {
  return `${wordA[i % wordA.length]} ${wordB[Math.floor(i / 6) % wordB.length]} ${i + 1}`;
}

let sort = 0;

function personEntityOp(i) {
  const id = personId(i);
  const name = displayName(i);
  const slug = name.toLowerCase().replaceAll(" ", ".");
  const tags = [...new Set(committeesFor[i].map((c) => COMMITTEE_TAG[c]))];
  const interestTags = tags.length > 0 ? tags : [COMMITTEE_TAG[i % COMMITTEE_TAG.length]];
  sort += 1;
  const attrs = {
    display_name: attribute({ type: "text", value: name }, "group", id, sort),
    tribe: attribute({ type: "text", value: FICTIONAL_TRIBES[i % FICTIONAL_TRIBES.length] }, "group", id, sort),
    role: attribute({ type: "text", value: ROLE_TITLES[i % ROLE_TITLES.length] }, "group", id, sort),
    areas_of_interest: attribute({ type: "tags", value: interestTags }, "group", id, sort),
    specialties: attribute({ type: "tags", value: interestTags.slice(0, 2) }, "group", id, sort),
    events_of_interest: attribute(
      { type: "tags", value: [EVENT_TAGS[i % EVENT_TAGS.length], EVENT_TAGS[(i + 1) % EVENT_TAGS.length]] },
      "group",
      id,
      sort,
    ),
    contact_email: attribute({ type: "link", value: { url: `mailto:${slug}@example.test`, format: "email" } }, "trusted", id, sort),
    contact_preference: attribute({ type: "enum", value: ["email", "phone", "no-contact"][i % 3] }, "group", id, sort),
  };
  return operation(sort, id, {
    op: "EntityCreate",
    entity: {
      id,
      group_id: groupId,
      kind: "person",
      attributes: attrs,
      owner: id,
      presence_visibility: groupVisibility,
      lifecycle: "active",
      provenance: provenance(id, sort),
      tier: tierT1,
      schema_version: schemaVersion,
    },
  });
}

function committeeEntityOp(c) {
  const id = committeeId(c);
  sort += 1;
  return operation(sort, facilitator, {
    op: "EntityCreate",
    entity: {
      id,
      group_id: groupId,
      kind: "committee",
      attributes: {
        display_name: attribute({ type: "text", value: COMMITTEES[c] }, "public", facilitator, sort),
      },
      owner: null,
      presence_visibility: groupVisibility,
      lifecycle: "active",
      provenance: provenance(facilitator, sort),
      tier: tierT1,
      schema_version: schemaVersion,
    },
  });
}

function organizationEntityOp(o) {
  const id = orgId(o);
  sort += 1;
  return operation(sort, facilitator, {
    op: "EntityCreate",
    entity: {
      id,
      group_id: groupId,
      kind: "organization",
      attributes: {
        display_name: attribute({ type: "text", value: ORG_NAMES[o] }, "public", facilitator, sort),
        org_type: attribute({ type: "enum", value: ORG_TYPES[o % ORG_TYPES.length] }, "public", facilitator, sort),
        website: attribute({ type: "link", value: { url: `https://example.test/org/${o + 1}`, format: "url" } }, "public", facilitator, sort),
      },
      owner: null,
      presence_visibility: groupVisibility,
      lifecycle: "active",
      provenance: provenance(facilitator, sort),
      tier: tierT1,
      schema_version: schemaVersion,
    },
  });
}

function edgeOp(id, kind, from, to, directed, weight, actor) {
  sort += 1;
  return operation(sort, actor, {
    op: "EdgeCreate",
    edge: {
      id,
      group_id: groupId,
      kind,
      from,
      to,
      directed,
      weight,
      attributes: {},
      visibility: groupVisibility,
      lifecycle: "active",
      provenance: provenance(actor, sort),
      tier: tierT1,
      schema_version: schemaVersion,
    },
  });
}

function governanceOp(personIndex) {
  const person = personId(personIndex);
  sort += 1;
  return operation(sort, person, {
    op: "MembershipAdd",
    membership: {
      id: uuid(940000 + sort),
      group_id: groupId,
      person,
      role: "governance",
      lifecycle: "active",
      provenance: provenance(person, sort),
      schema_version: schemaVersion,
    },
  });
}

// --- Assemble ---
const template = JSON.parse(
  readFileSync(path.join(fixturesDir, "templates", "atni-convention.template.json"), "utf8"),
);

const ops = [];
sort += 1;
ops.push(
  operation(sort, facilitator, {
    op: "GroupCreate",
    group: {
      id: groupId,
      name: template.name,
      template_id: template.template_id,
      template_version: schemaVersion,
      provenance: provenance(facilitator, sort),
      tier: tierT1,
      schema_version: schemaVersion,
    },
    template_json: JSON.stringify(template),
  }),
);

for (const idx of [12, 13, 14]) {
  ops.push(governanceOp(idx));
}
for (let c = 0; c < COMMITTEE_COUNT; c += 1) {
  ops.push(committeeEntityOp(c));
}
for (let o = 0; o < ORG_COUNT; o += 1) {
  ops.push(organizationEntityOp(o));
}
for (let i = 0; i < PEOPLE_COUNT; i += 1) {
  ops.push(personEntityOp(i));
}

let memberOfIndex = 0;
for (let i = 0; i < PEOPLE_COUNT; i += 1) {
  for (const c of committeesFor[i]) {
    memberOfIndex += 1;
    ops.push(edgeOp(uuid(950000 + memberOfIndex), "member_of", personId(i), committeeId(c), true, null, personId(i)));
  }
}

let affiliatedIndex = 0;
for (let i = 0; i < PEOPLE_COUNT; i += 1) {
  for (const o of orgsFor[i]) {
    affiliatedIndex += 1;
    ops.push(edgeOp(uuid(960000 + affiliatedIndex), "affiliated_with", personId(i), orgId(o), true, null, personId(i)));
  }
}

const connectedPairs = new Set();
let connectedIndex = 0;
for (let i = 0; i < PEOPLE_COUNT && connectedIndex < 45; i += 1) {
  const j = (i * 13 + 7) % PEOPLE_COUNT;
  if (j === i) {
    continue;
  }
  const key = i < j ? `${i}-${j}` : `${j}-${i}`;
  if (connectedPairs.has(key)) {
    continue;
  }
  connectedPairs.add(key);
  connectedIndex += 1;
  ops.push(edgeOp(uuid(970000 + connectedIndex), "connected_to", personId(i), personId(j), false, null, personId(i)));
}

mkdirSync(outDir, { recursive: true });
writeFileSync(
  path.join(outDir, "atni-convention.ops.jsonl"),
  ops.map((op) => JSON.stringify(op)).join("\n") + "\n",
);
console.log(`Wrote ${ops.length} ops to fixtures/groups/atni-convention.ops.jsonl`);
console.log(`People: ${PEOPLE_COUNT}, committees: ${COMMITTEE_COUNT}, organizations: ${ORG_COUNT}`);
console.log(`member_of: ${memberOfIndex}, affiliated_with: ${affiliatedIndex}, connected_to: ${connectedIndex}`);
