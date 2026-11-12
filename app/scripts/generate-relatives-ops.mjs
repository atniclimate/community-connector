// Generates fixtures/groups/relatives-demo.ops.jsonl: a FULLY SYNTHETIC group
// for person-centered and continuance features (seats that outlive holders,
// more-than-human relatives, capacity and rest, introductions). Every name is
// fictional; no emails, phone numbers, real Nation names, or real people (I1).
// Deterministic: fixed index arithmetic only, no Math.random, no wall clock.
// The planted structures (fixtures/README.md) are asserted at the end, so a
// regeneration that breaks one fails loudly instead of shipping silently.
import assert from "node:assert/strict";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const here = path.dirname(fileURLToPath(import.meta.url));
const fixturesDir = path.resolve(here, "../../fixtures");
const outDir = path.join(fixturesDir, "groups");

const schemaVersion = "0.1.0";

function uuid(n) {
  const hex = BigInt(n).toString(16).padStart(32, "0");
  return `${hex.slice(0, 8)}-${hex.slice(8, 12)}-${hex.slice(12, 16)}-${hex.slice(16, 20)}-${hex.slice(20)}`;
}

const groupId = uuid(1200000);
const personId = (i) => uuid(1210000 + i + 1);
const orgId = (o) => uuid(1220000 + o + 1);
const seatId = (s) => uuid(1230000 + s + 1);
const placeId = (p) => uuid(1240000 + p + 1);
const relativeId = (r) => uuid(1250000 + r + 1);

// --- Organizations: [name, org_type, member person indices] ---
const ORGS = [
  ["Alder Tribal Natural Resources Dept", "tribal_department", [0, 1, 2, 3, 4, 5]],
  ["Birch Tribal Fisheries Dept", "tribal_department", [6, 7, 8, 9, 10, 11]],
  ["Cedar Tribal Environmental Dept", "tribal_department", [12, 13, 14, 15, 16, 17]],
  ["Fern Intertribal Council", "intertribal_body", [18, 19, 20, 21]],
  ["Harbor Water Agency", "agency", [29, 30, 31, 32]],
  ["Juniper Air Agency", "agency", [33, 34, 35]],
  ["Kestrel University", "university", [22, 23, 24, 25]],
  ["Lumen Watershed Nonprofit", "nonprofit", [26, 27, 28]],
];
const PEOPLE_COUNT = 36;

// --- Planted structures (documented by id in fixtures/README.md) ---
const FACILITATOR = 0;
const GOVERNANCE = 1;
const LIAISON = 3; // (a) only tie between the Alder dept and the agency cluster
const AGENCY_HUB = 29; // (a) agency-side end of the liaison's bridge
const PAIR_YES = { seeker: 8, introducer: 20, offerer: 24, tag: "eelgrass-survey-methods" }; // (b)
const PAIR_NO = { seeker: 14, introducer: 21, offerer: 27, tag: "grant-reporting-help" }; // (b)
const KNOWLEDGE_HOLDER = 10; // (c)
const SOLE_LEARNER = 11; // (c)
const SOLE_SPECIALTY = "weir-building";
const RESTING = 16; // (d)
const SEAT_PREVIOUS_HOLDER = 7; // (e) archived holds_seat
const SEAT_CURRENT_HOLDER = 9; // (e) active holds_seat
const BRIDGE_PEOPLE = [20, 24, 27]; // offering at network, presence at network
const TIER_CAPPED = 25; // T2 entity whose offering asks for network (ceiling: group)
const NOT_WILLING = new Set([PAIR_NO.introducer, 5, 13, 31]);

const FIRST = ["Alder", "Birch", "Cedar", "Drift", "Ember", "Fern", "Harbor", "Juniper", "Kestrel", "Lumen", "Moss", "Pine"];
const LAST = ["Riverbend", "Meadow", "Northstar", "Willow", "Hearth", "Stone"];
const SPECIALTIES = ["stream-surveys", "gis-mapping", "grant-writing", "youth-programs", "water-sampling", "plant-propagation", "outreach", "data-stewardship"];
const ASKS = ["field-volunteers", "mapping-support", "data-sharing-agreements", "youth-mentoring", "meeting-facilitation", "water-sampling", "native-plant-seed", "culvert-assessment"];

// [title, org index, current holder, last_confirmed]
const SEATS = [
  ["Fisheries Policy Representative", 1, SEAT_CURRENT_HOLDER, "2026-10-01"],
  ["Water Quality Lead", 0, 4, "2026-06-01"],
  ["Climate Delegate", 3, 18, "2026-05-01"],
  ["Tribal Liaison Officer", 4, 30, "2026-04-01"],
  ["Research Partnerships Coordinator", 6, 22, "2026-03-01"],
  ["Restoration Program Manager", 7, 26, "2026-02-01"],
];
// [name, place_type, geo point or null]
const PLACES = [
  ["Alder Creek Watershed", "watershed", { lat: 46.5, lon: -121.5 }],
  ["Birch River", "river", { lat: 46.25, lon: -122 }],
  ["Juniper Valley Airshed", "airshed", null],
  ["Fern Meadow", "meadow", null],
];
// [name, relative_type, seasons, lives_at place index]
const RELATIVES = [
  ["Spring salmon run", "fish_run", ["spring", "summer"], 1],
  ["Cedar grove", "tree", ["year-round"], 0],
  ["Huckleberry meadow", "plant", ["late-summer"], 3],
];

const orgOf = (i) => ORGS.findIndex(([, , members]) => members.includes(i));

let sort = 0;

function provenance(actor, at) {
  return { origin: "authored", recorded_by: { human: actor }, responsible_human: actor, recorded_at: at, custody: [], schema_version: schemaVersion };
}

function operation(actor, kind) {
  sort += 1;
  const opId = uuid(1280000 + sort);
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

function attrs(actor, values, visibilities = {}) {
  const out = {};
  for (const [id, value] of Object.entries(values)) {
    out[id] = { value, visibility: visibilities[id] ?? "group", tier_override: null, provenance: provenance(actor, sort + 1) };
  }
  return out;
}

function entityOp({ id, kind, owner, actor, values, visibilities, presence = "group", tier = "T1" }) {
  return operation(actor, {
    op: "EntityCreate",
    entity: {
      id,
      group_id: groupId,
      kind,
      attributes: attrs(actor, values, visibilities),
      owner,
      presence_visibility: presence,
      lifecycle: "active",
      provenance: provenance(actor, sort + 1),
      tier,
      schema_version: schemaVersion,
    },
  });
}

const tags = (value) => ({ type: "tags", value });
const text = (value) => ({ type: "text", value });
const enumValue = (value) => ({ type: "enum", value });
const date = (value) => ({ type: "date", value });

function personValues(i) {
  const specialties = [SPECIALTIES[i % SPECIALTIES.length], SPECIALTIES[(i * 3 + 1) % SPECIALTIES.length]];
  if (i === KNOWLEDGE_HOLDER) {
    specialties[1] = SOLE_SPECIALTY;
  }
  const seeking = [ASKS[(i * 3) % ASKS.length]];
  const offering = [ASKS[(i * 5 + 1) % ASKS.length]];
  for (const pair of [PAIR_YES, PAIR_NO]) {
    if (i === pair.seeker) seeking.push(pair.tag);
    if (i === pair.offerer) offering.push(pair.tag);
  }
  const values = {
    display_name: text(`${FIRST[i % FIRST.length]} ${LAST[Math.floor(i / 6) % LAST.length]} ${i + 1}`),
    nation_or_org: text(ORGS[orgOf(i)][0]),
    seeking: tags(seeking),
    offering: tags(offering),
    specialties: tags(specialties),
    willing_to_introduce: enumValue(NOT_WILLING.has(i) ? "no" : "yes"),
    capacity: enumValue(i === RESTING ? "resting" : i % 3 === 0 ? "limited" : "open"),
  };
  if (i === RESTING) {
    values.resting_until = date("2027-03-01");
  }
  return values;
}

function personOp(i) {
  const id = personId(i);
  const bridge = BRIDGE_PEOPLE.includes(i);
  const capped = i === TIER_CAPPED;
  return entityOp({
    id,
    kind: "person",
    owner: id,
    actor: id,
    values: personValues(i),
    visibilities: bridge || capped ? { offering: "network" } : {},
    presence: bridge ? "network" : "group",
    tier: capped ? "T2" : "T1",
  });
}

function membershipOp(i, role) {
  const person = personId(i);
  return operation(person, {
    op: "MembershipAdd",
    membership: {
      id: uuid(1270000 + sort + 1),
      group_id: groupId,
      person,
      role,
      lifecycle: "active",
      provenance: provenance(person, sort + 1),
      schema_version: schemaVersion,
    },
  });
}

const edges = [];
function edgeOp(kind, from, to, directed, actor) {
  const id = uuid(1260000 + edges.length + 1);
  edges.push({ id, kind, from, to, active: true });
  return operation(actor, {
    op: "EdgeCreate",
    edge: {
      id,
      group_id: groupId,
      kind,
      from,
      to,
      directed,
      weight: null,
      attributes: {},
      visibility: "group",
      lifecycle: "active",
      provenance: provenance(actor, sort + 1),
      tier: "T1",
      schema_version: schemaVersion,
    },
  });
}

const facilitator = personId(FACILITATOR);
const person = (i) => personId(i);
const ops = [];

function buildEntities(template) {
  ops.push(operation(facilitator, {
    op: "GroupCreate",
    group: { id: groupId, name: template.name, template_id: template.template_id, template_version: schemaVersion, provenance: provenance(facilitator, sort + 1), tier: "T1", schema_version: schemaVersion },
    template_json: JSON.stringify(template),
  }));
  for (let i = 0; i < PEOPLE_COUNT; i += 1) {
    ops.push(membershipOp(i, "member"));
  }
  ops.push(membershipOp(FACILITATOR, "facilitator"), membershipOp(GOVERNANCE, "governance"));
  const nonPerson = (id, kind, values) => entityOp({ id, kind, owner: null, actor: facilitator, values });
  ORGS.forEach(([name, type], o) => ops.push(nonPerson(orgId(o), "organization", { display_name: text(name), org_type: enumValue(type) })));
  SEATS.forEach(([title, , , confirmed], s) => ops.push(nonPerson(seatId(s), "seat", { title: text(title), last_confirmed: date(confirmed) })));
  PLACES.forEach(([name, type, point], p) => {
    const values = { display_name: text(name), place_type: enumValue(type) };
    if (point !== null) values.location = { type: "geo", value: { point } };
    ops.push(nonPerson(placeId(p), "place", values));
  });
  RELATIVES.forEach(([name, type, seasons], r) => ops.push(nonPerson(relativeId(r), "relative", { display_name: text(name), relative_type: enumValue(type), seasons: tags(seasons) })));
  for (let i = 0; i < PEOPLE_COUNT; i += 1) {
    ops.push(personOp(i));
  }
}

// connected_to: a chain inside each organization, then the cross ties below.
// The tribal and agency sides meet only at LIAISON - AGENCY_HUB.
const CROSS_TIES = [
  [5, 6], [11, 12], [17, 18], [1, 19], [19, 22], [25, 26], [32, 33],
  [PAIR_YES.seeker, PAIR_YES.introducer], [PAIR_YES.introducer, PAIR_YES.offerer],
  [PAIR_NO.seeker, PAIR_NO.introducer], [PAIR_NO.introducer, PAIR_NO.offerer],
  [LIAISON, AGENCY_HUB],
];
// learned_from [learner, teacher]; KNOWLEDGE_HOLDER teaches exactly one learner.
const LEARNED = [[SOLE_LEARNER, KNOWLEDGE_HOLDER], [13, 12], [23, 22], [31, 30], [2, 0], [19, 18]];
// cares_for [from id, to id]
const CARES = [
  [orgId(1), relativeId(0)], [orgId(0), placeId(0)], [orgId(2), relativeId(1)], [orgId(3), placeId(1)],
  [orgId(7), placeId(0)], [orgId(2), placeId(3)], [person(12), relativeId(2)], [person(6), relativeId(0)],
  [orgId(5), placeId(2)],
];

function buildEdges() {
  ORGS.forEach(([, , members], o) => {
    for (const i of members) ops.push(edgeOp("member_of", person(i), orgId(o), true, person(i)));
    for (let k = 1; k < members.length; k += 1) {
      ops.push(edgeOp("connected_to", person(members[k - 1]), person(members[k]), false, person(members[k - 1])));
    }
  });
  for (const [a, b] of CROSS_TIES) ops.push(edgeOp("connected_to", person(a), person(b), false, person(a)));
  for (const [learner, teacher] of LEARNED) ops.push(edgeOp("learned_from", person(learner), person(teacher), true, person(learner)));
  SEATS.forEach(([, o, holder], s) => {
    ops.push(edgeOp("seat_in", seatId(s), orgId(o), true, facilitator));
    if (s === 0) {
      ops.push(edgeOp("holds_seat", person(SEAT_PREVIOUS_HOLDER), seatId(s), true, facilitator));
    }
    ops.push(edgeOp("holds_seat", person(holder), seatId(s), true, facilitator));
  });
  for (const [from, to] of CARES) ops.push(edgeOp("cares_for", from, to, true, facilitator));
  RELATIVES.forEach(([, , , p], r) => ops.push(edgeOp("lives_at", relativeId(r), placeId(p), true, facilitator)));
  // (e) the previous holder steps down after the seat was handed on.
  const previous = edges.find((e) => e.kind === "holds_seat" && e.from === person(SEAT_PREVIOUS_HOLDER));
  previous.active = false;
  ops.push(operation(facilitator, { op: "EdgeLifecycleSet", edge: previous.id, lifecycle: "archived" }));
}

// --- Self-check of the planted structures over active edges ---
function neighbors(skipNode = null, skipEdge = null) {
  const adj = new Map();
  const add = (a, b) => adj.set(a, [...(adj.get(a) ?? []), b]);
  for (const e of edges) {
    if (!e.active || e.id === skipEdge || e.from === skipNode || e.to === skipNode) continue;
    add(e.from, e.to);
    add(e.to, e.from);
  }
  return adj;
}

function reaches(adj, start, goal) {
  const seen = new Set([start]);
  const queue = [start];
  while (queue.length > 0) {
    const node = queue.shift();
    if (node === goal) return true;
    for (const next of adj.get(node) ?? []) {
      if (!seen.has(next)) {
        seen.add(next);
        queue.push(next);
      }
    }
  }
  return false;
}

function checkPlanted() {
  const hub = person(AGENCY_HUB);
  const tribal = person(0);
  assert.ok(reaches(neighbors(), hub, tribal), "(a) graph should be connected through the liaison");
  assert.ok(!reaches(neighbors(person(LIAISON)), hub, tribal), "(a) liaison must be an articulation point");
  const bridge = edges.find((e) => e.from === person(LIAISON) && e.to === hub);
  assert.ok(!reaches(neighbors(null, bridge.id), hub, tribal), "(a) liaison-agency tie must be a bridge");
  const adj = neighbors();
  for (const pair of [PAIR_YES, PAIR_NO]) {
    const seekerNeighbors = new Set(adj.get(person(pair.seeker)));
    assert.ok(!seekerNeighbors.has(person(pair.offerer)), "(b) pair must not be adjacent");
    const mutual = [...new Set(adj.get(person(pair.offerer)))].filter((n) => seekerNeighbors.has(n));
    assert.deepEqual(mutual, [person(pair.introducer)], "(b) exactly one mutual neighbor");
  }
  const holders = Array.from({ length: PEOPLE_COUNT }, (_, i) => i).filter((i) => personValues(i).specialties.value.includes(SOLE_SPECIALTY));
  assert.deepEqual(holders, [KNOWLEDGE_HOLDER], "(c) sole knowledge holder");
  const learners = edges.filter((e) => e.kind === "learned_from" && e.to === person(KNOWLEDGE_HOLDER));
  assert.deepEqual(learners.map((e) => e.from), [person(SOLE_LEARNER)], "(c) exactly one learner");
  const seatEdges = edges.filter((e) => e.kind === "holds_seat" && e.to === seatId(0));
  assert.deepEqual(seatEdges.map((e) => e.active), [false, true], "(e) one archived and one current holder");
}

const template = JSON.parse(readFileSync(path.join(fixturesDir, "templates", "relatives-demo.template.json"), "utf8"));
buildEntities(template);
buildEdges();
checkPlanted();

mkdirSync(outDir, { recursive: true });
writeFileSync(path.join(outDir, "relatives-demo.ops.jsonl"), ops.map((op) => JSON.stringify(op)).join("\n") + "\n");
const entityCount = PEOPLE_COUNT + ORGS.length + SEATS.length + PLACES.length + RELATIVES.length;
console.log(`Wrote ${ops.length} ops to fixtures/groups/relatives-demo.ops.jsonl`);
console.log(`Entities: ${entityCount}, edges: ${edges.length} (${edges.filter((e) => e.active).length} active)`);
