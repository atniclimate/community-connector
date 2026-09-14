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
  /** EntityCreate count per entity kind. */
  readonly countByKind: ReadonlyMap<string, number>;
  /** Distinct entities incident to at least one connected_to edge. */
  readonly connectedPeople: number;
};

const TIE_EDGE_KIND = "connected_to";
const PERSON_KIND = "person";

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
  const countByKind = new Map<string, number>();
  const tied = new Set<string>();
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
        countByKind.set(entity["kind"], (countByKind.get(entity["kind"]) ?? 0) + 1);
      }
    } else if (kind["op"] === "EdgeCreate" && isObject(kind["edge"])) {
      const edge = kind["edge"];
      if (typeof edge["kind"] === "string") {
        edgeKinds.add(edge["kind"]);
      }
      if (edge["kind"] === TIE_EDGE_KIND && typeof edge["from"] === "string" && typeof edge["to"] === "string") {
        tied.add(edge["from"]);
        tied.add(edge["to"]);
      }
    }
  }
  return { entityIds, entityKinds, edgeKinds, countByKind, connectedPeople: tied.size };
}

/** Every integer that appears in a caption, in order of appearance. */
function captionNumbers(label: string): readonly number[] {
  return [...label.matchAll(/\d+/g)].map((match) => Number.parseInt(match[0], 10));
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

  it("gates labels on every beat to kinds in the fixture, never person (D-099: no person name on stage)", () => {
    // The sheet is presented on a stage; every beat must carry the gate, so a
    // beat added without one fails here instead of naming someone on screen.
    for (const beat of beats) {
      expect(beat.labelKinds, `${beat.id}: labelKinds missing`).toBeDefined();
      expect(beat.labelKinds, `${beat.id}: labelKinds must name at least one kind`).not.toHaveLength(0);
      for (const kind of beat.labelKinds ?? []) {
        expect(fixture.entityKinds.has(kind), `${beat.id}: label kind ${kind}`).toBe(true);
        expect(kind, `${beat.id}: person labels are never rendered on stage`).not.toBe(PERSON_KIND);
      }
    }
  });

  it("states only counts the fixture makes true (48 tied people; 60 / 15 / 12 per kind)", () => {
    const perKind = (kind: string): number => fixture.countByKind.get(kind) ?? 0;
    // Every number any caption states must be one of these fixture-derived
    // truths, so a regenerated fixture cannot drift from a caption unnoticed.
    const truths = new Set([fixture.connectedPeople, ...fixture.countByKind.values()]);
    for (const beat of beats) {
      for (const number of captionNumbers(beat.label)) {
        expect(truths.has(number), `${beat.id}: caption states ${number}, fixture supports ${[...truths].join(", ")}`)
          .toBe(true);
      }
    }
    const priorities = beats.find((beat) => beat.id === "shared-priorities");
    if (priorities !== undefined) {
      expect(captionNumbers(priorities.label)).toEqual([fixture.connectedPeople]);
      expect(priorities.filter?.edgeKinds).toEqual([TIE_EDGE_KIND]);
    }
    const finale = beats.find((beat) => beat.id === "constellation");
    if (finale !== undefined) {
      expect(captionNumbers(finale.label)).toEqual([perKind(PERSON_KIND), perKind("committee"), perKind("organization")]);
    }
    // The values the 2026-09-15 sheet was authored against; a regenerated
    // fixture that changes them must re-author the captions deliberately.
    expect(fixture.connectedPeople).toBe(48);
    expect(perKind(PERSON_KIND)).toBe(60);
    expect(perKind("committee")).toBe(15);
    expect(perKind("organization")).toBe(12);
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
