import { describe, expect, it } from "vitest";
import type { JsonObject, PresentBeat } from "../state/state";
import { measureHighlights, presenterBeatText } from "./presenter";

const response: JsonObject = {
  betweenness: {
    "entity-a": { entity: "entity-a", value: 0.2, explanation: "connects some shortest paths" },
    "entity-b": { entity: "entity-b", value: 0.8, explanation: "connects most shortest paths" },
    "entity-c": { entity: "entity-c", value: 0.2, explanation: "connects tied shortest paths" },
  },
  degree: {
    "entity-a": { entity: "entity-a", degree: 1, single_tie: true, explanation: "connected to 1 other person" },
    "entity-b": { entity: "entity-b", degree: 3, single_tie: false, explanation: "connected to 3 others" },
    "entity-c": { entity: "entity-c", degree: 1, single_tie: true, explanation: "has one connection" },
  },
};

describe("presenter measure highlights", () => {
  it("selects top betweenness values with deterministic id tie-breaking", () => {
    const beat: PresentBeat = {
      id: "bridges",
      label: "Network bridges",
      measure: "betweenness_top_n",
      topN: 2,
    };
    const highlights = measureHighlights(response, beat);

    expect([...highlights.ids]).toEqual(["entity-b", "entity-a"]);
    expect(highlights.explanations).toEqual([
      "connects most shortest paths",
      "connects some shortest paths",
    ]);
    expect(presenterBeatText(beat, highlights.explanations)).toBe(
      "Network bridges: connects most shortest paths; connects some shortest paths",
    );
  });

  it("selects every single-tie entity and excludes all other degree records", () => {
    const beat: PresentBeat = { id: "single-ties", label: "Single ties", measure: "single_tie" };
    const highlights = measureHighlights(response, beat);

    expect([...highlights.ids]).toEqual(["entity-a", "entity-c"]);
    expect(highlights.explanations).toEqual([
      "connected to 1 other person",
      "has one connection",
    ]);
  });

  it("rejects malformed measure responses so callers can surface the failure", () => {
    const beat: PresentBeat = { id: "bridges", label: "Network bridges", measure: "betweenness_top_n", topN: 1 };

    expect(() => measureHighlights({ betweenness: [] }, beat)).toThrow(
      "Graph measures response is missing betweenness",
    );
  });
});
