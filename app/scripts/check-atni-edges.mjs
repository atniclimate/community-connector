// Verifies the CS-02 (D-103 pick 2) invariant on the ATNI convention fixture:
// every connected_to edge's endpoints share at least two areas_of_interest
// tags, and prints per-edge-kind counts plus the spotlight person's neighbor
// count. Exits nonzero on any violation.
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const here = path.dirname(fileURLToPath(import.meta.url));
const repo = path.resolve(here, "../..");
const fixturePath = path.join(repo, "fixtures", "groups", "atni-convention.ops.jsonl");

const SHARED_TAG_THRESHOLD = 2;
// Must match SPOTLIGHT_PERSON_INDEX's id in app/scripts/generate-atni-ops.mjs.
const SPOTLIGHT_PERSON_ID = "00000000-0000-0000-0000-0000000de2c5";

const lines = readFileSync(fixturePath, "utf8").trim().split("\n").filter(Boolean);
const ops = lines.map((line) => JSON.parse(line));

const people = new Map(); // id -> areas_of_interest tags
const edgesByKind = new Map();

for (const op of ops) {
  const k = op.kind;
  if (!k) continue;
  if (k.op === "EntityCreate" && k.entity.kind === "person") {
    const tags = k.entity.attributes.areas_of_interest?.value?.value ?? [];
    people.set(k.entity.id, tags);
  }
  if (k.op === "EdgeCreate") {
    const kind = k.edge.kind;
    const list = edgesByKind.get(kind) ?? [];
    list.push(k.edge);
    edgesByKind.set(kind, list);
  }
}

if (!people.has(SPOTLIGHT_PERSON_ID)) {
  console.error(`FAIL: spotlight person ${SPOTLIGHT_PERSON_ID} not found among EntityCreate people`);
  process.exit(1);
}

const connectedEdges = edgesByKind.get("connected_to") ?? [];
let violations = 0;
let spotlightNeighbors = 0;

for (const edge of connectedEdges) {
  const fromTags = people.get(edge.from) ?? [];
  const toTags = people.get(edge.to) ?? [];
  const shared = fromTags.filter((tag) => toTags.includes(tag)).length;
  if (shared < SHARED_TAG_THRESHOLD) {
    violations += 1;
    console.error(
      `FAIL: connected_to ${edge.from} <-> ${edge.to} shares only ${shared} tag(s) ` +
        `(need >= ${SHARED_TAG_THRESHOLD}): [${fromTags.join(", ")}] vs [${toTags.join(", ")}]`,
    );
  }
  if (edge.from === SPOTLIGHT_PERSON_ID || edge.to === SPOTLIGHT_PERSON_ID) {
    spotlightNeighbors += 1;
  }
}

console.log("Edge counts by kind:");
for (const [kind, list] of [...edgesByKind.entries()].sort(([a], [b]) => a.localeCompare(b))) {
  console.log(`  ${kind}: ${list.length}`);
}
console.log(`Spotlight person ${SPOTLIGHT_PERSON_ID} connected_to neighbors: ${spotlightNeighbors}`);

if (spotlightNeighbors < 4 || spotlightNeighbors > 10) {
  violations += 1;
  console.error(`FAIL: spotlight neighbor count ${spotlightNeighbors} outside the required [4, 10] range`);
}

if (connectedEdges.length < 40 || connectedEdges.length > 90) {
  violations += 1;
  console.error(`FAIL: connected_to edge count ${connectedEdges.length} outside the required [40, 90] range`);
}

if (violations > 0) {
  console.error(`\n${violations} violation(s) found.`);
  process.exit(1);
}

console.log("\nOK: every connected_to edge shares >= 2 areas_of_interest tags; spotlight and total counts in range.");
