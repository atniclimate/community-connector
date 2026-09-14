import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";
import type { JsonObject, JsonValue, PresentBeat, PresentCamera, PresentMeasure } from "../state/state";

// Validates the committed ATNI beat sheet against the committed synthetic
// fixture it presents (D-103: "a mistyped id fails silently" is closed here).
// Both files are read from disk relative to this test; nothing is fetched.

const here = path.dirname(fileURLToPath(import.meta.url));
const beatsPath = path.resolve(here, "../../public/beats.atni.json");
const opsPath = path.resolve(here, "../../../fixtures/groups/atni-convention.ops.jsonl");

const MEASURES: readonly PresentMeasure[] = ["betweenness_top_n", "single_tie"];
const CAMERAS: readonly PresentCamera[] = ["hold", "fit", "fly"];

type FixtureIndex = {
  readonly entityIds: ReadonlySet<string>;
  readonly entityKinds: ReadonlySet<string>;
  readonly edgeKinds: ReadonlySet<string>;
};

function isObject(value: JsonValue | undefined): value is JsonObject {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function loadBeats(): readonly PresentBeat[] {
  const parsed: unknown = JSON.parse(readFileSync(beatsPath, "utf8"));
  if (!Array.isArray(parsed)) {
    throw new Error("beats.atni.json must be a JSON array of beats");
  }
  return parsed as readonly PresentBeat[];
}

function loadFixture(): FixtureIndex {
  const entityIds = new Set<string>();
  const entityKinds = new Set<string>();
  const edgeKinds = new Set<string>();
  const lines = readFileSync(opsPath, "utf8").split("\n").filter((line) => line.trim() !== "");
  for (const line of lines) {
    const op: unknown = JSON.parse(line);
    if (!isObject(op as JsonValue) || !isObject((op as JsonObject)["kind"])) {
      continue;
    }
    const kind = (op as JsonObject)["kind"] as JsonObject;
    if (kind["op"] === "EntityCreate" && isObject(kind["entity"])) {
      const entity = kind["entity"];
      if (typeof entity["id"] === "string") {
        entityIds.add(entity["id"]);
      }
      if (typeof entity["kind"] === "string") {
        entityKinds.add(entity["kind"]);
      }
    } else if (kind["op"] === "EdgeCreate" && isObject(kind["edge"])) {
      const edge = kind["edge"];
      if (typeof edge["kind"] === "string") {
        edgeKinds.add(edge["kind"]);
      }
    }
  }
  return { entityIds, entityKinds, edgeKinds };
}

const beats = loadBeats();
const fixture = loadFixture();

describe("beats.atni.json against the atni-convention fixture", () => {
  it("reads a non-empty beat sheet and a fixture with entities and edges", () => {
    expect(beats.length).toBeGreaterThan(0);
    expect(fixture.entityIds.size).toBeGreaterThan(0);
    expect(fixture.entityKinds.size).toBeGreaterThan(0);
    expect(fixture.edgeKinds.size).toBeGreaterThan(0);
  });

  it("gives every beat a unique string id and a string label", () => {
    const ids = beats.map((beat) => beat.id);
    for (const beat of beats) {
      expect(typeof beat.id).toBe("string");
      expect(beat.id).not.toBe("");
      expect(typeof beat.label).toBe("string");
      expect(beat.label).not.toBe("");
    }
    expect(new Set(ids).size).toBe(ids.length);
  });

  it("resolves every focusEntityId to an EntityCreate id in the fixture", () => {
    for (const beat of beats) {
      if (beat.focusEntityId !== undefined) {
        expect(fixture.entityIds.has(beat.focusEntityId), `${beat.id}: focusEntityId ${beat.focusEntityId}`).toBe(true);
      }
    }
  });

  it("uses only PresentMeasure members, with a positive integer topN for betweenness", () => {
    for (const beat of beats) {
      if (beat.measure === undefined) {
        expect(beat.topN, `${beat.id}: topN without a measure`).toBeUndefined();
        continue;
      }
      expect(MEASURES, `${beat.id}: measure ${String(beat.measure)}`).toContain(beat.measure);
      if (beat.measure === "betweenness_top_n") {
        expect(Number.isInteger(beat.topN), `${beat.id}: topN ${String(beat.topN)}`).toBe(true);
        expect(beat.topN ?? 0).toBeGreaterThan(0);
      }
    }
  });

  it("uses only camera values the presenter understands", () => {
    for (const beat of beats) {
      if (beat.camera !== undefined) {
        expect(CAMERAS, `${beat.id}: camera ${String(beat.camera)}`).toContain(beat.camera);
      }
    }
  });

  it("filters only on entity kinds and edge kinds that exist in the fixture", () => {
    for (const beat of beats) {
      for (const kind of beat.filter?.kinds ?? []) {
        expect(fixture.entityKinds.has(kind), `${beat.id}: entity kind ${kind}`).toBe(true);
      }
      for (const edgeKind of beat.filter?.edgeKinds ?? []) {
        expect(fixture.edgeKinds.has(edgeKind), `${beat.id}: edge kind ${edgeKind}`).toBe(true);
      }
      if (beat.filter !== undefined) {
        expect((beat.filter.kinds?.length ?? 0) + (beat.filter.edgeKinds?.length ?? 0), `${beat.id}: empty filter`)
          .toBeGreaterThan(0);
      }
    }
  });

  it("carries an all-lit beat, and once the constellation finale exists it is last (D-103.4)", () => {
    const allLit = (beat: PresentBeat): boolean =>
      beat.filter === undefined && beat.measure === undefined && beat.focusEntityId === undefined;
    // Today's pre-A9 sheet opens on the all-lit overview and ends on a kind
    // beat; the CS-04 sheet ends on `constellation`. Both must satisfy this.
    expect(beats.some(allLit), "no beat lights the whole graph").toBe(true);
    const finaleIndex = beats.findIndex((beat) => beat.id === "constellation");
    if (finaleIndex !== -1) {
      const finale = beats[finaleIndex];
      expect(finaleIndex, "constellation must be the last beat").toBe(beats.length - 1);
      expect(finale?.filter).toBeUndefined();
      expect(finale?.measure).toBeUndefined();
      expect(finale?.focusEntityId).toBeUndefined();
    }
  });
});
