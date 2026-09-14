import { describe, expect, it } from "vitest";
import type { JsonObject, PresentBeat, ProjectionDto } from "../state/state";
import { computeFocusSet } from "./focus";
import {
  beatCameraMove,
  beatHighlights,
  beatIndexForKey,
  edgeKindBeatHighlights,
  kindBeatHighlights,
  measureHighlights,
  presenterBeatText,
  type CameraMoveContext,
} from "./presenter";

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

// Synthetic projection: two people tied by a shared-priority edge, each a
// member of a committee, one affiliated with an organization.
const projection: ProjectionDto = {
  entities: [
    { id: "p1", kind: "person" },
    { id: "p2", kind: "person" },
    { id: "p3", kind: "person" },
    { id: "c1", kind: "committee" },
    { id: "o1", kind: "organization" },
  ],
  edges: [
    { id: "e-tie", kind: "connected_to", from: "p1", to: "p2" },
    { id: "e-m1", kind: "member_of", from: "p1", to: "c1" },
    { id: "e-m3", kind: "member_of", from: "p3", to: "c1" },
    { id: "e-aff", kind: "affiliated_with", from: "p2", to: "o1" },
  ],
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

  it("foregrounds a kind beat's kinds and leaves measure beats to the measure call", () => {
    const entities = [
      { id: "p1", kind: "person" },
      { id: "c1", kind: "committee" },
      { id: "c2", kind: "committee" },
    ];

    expect([...(kindBeatHighlights(entities, { id: "c", label: "Committees", filter: { kinds: ["committee"] } }) ?? [])])
      .toEqual(["c1", "c2"]);
    expect(kindBeatHighlights(entities, { id: "all", label: "Overview" })?.size).toBe(0);
    expect(kindBeatHighlights(entities, undefined)?.size).toBe(0);
    expect(kindBeatHighlights(entities, { id: "b", label: "Bridges", measure: "betweenness_top_n", topN: 3 })).toBeNull();
  });

  it("rejects malformed measure responses so callers can surface the failure", () => {
    const beat: PresentBeat = { id: "bridges", label: "Network bridges", measure: "betweenness_top_n", topN: 1 };

    expect(() => measureHighlights({ betweenness: [] }, beat)).toThrow(
      "Graph measures response is missing betweenness",
    );
  });
});

describe("presenter captions (D-103.5: counts, never names)", () => {
  it("shows the label alone when there is nothing to count", () => {
    expect(presenterBeatText({ id: "all", label: "Overview" }, [])).toBe("Overview");
    expect(presenterBeatText({ id: "m", label: "Members", filter: { kinds: ["person"] } }, [], 60)).toBe("Members");
    expect(presenterBeatText(undefined, ["anything"])).toBe("");
  });

  it("shows a count on measure beats and never concatenates explanations", () => {
    const beat: PresentBeat = { id: "bridges", label: "Network bridges", measure: "betweenness_top_n", topN: 2 };
    const highlights = measureHighlights(response, beat);
    const text = presenterBeatText(beat, highlights.explanations, highlights.ids.size);

    expect(text).toBe("Network bridges - 2 highlighted");
    expect(text).not.toContain("shortest paths");
    expect(presenterBeatText(beat, highlights.explanations)).toBe("Network bridges - 2 highlighted");
  });
});

describe("presenter edge-kind highlights (D-103 shared-priorities beat)", () => {
  it("lights every entity incident to an edge of the given kinds and keeps those edges bright", () => {
    const beat: PresentBeat = { id: "ties", label: "Shared priorities", filter: { edgeKinds: ["connected_to"] } };
    const highlights = edgeKindBeatHighlights(projection.edges, beat);

    expect(highlights).not.toBeNull();
    expect([...(highlights?.ids ?? [])].sort()).toEqual(["p1", "p2"]);
    expect([...(highlights?.edgeIds ?? [])]).toEqual(["e-tie"]);
  });

  it("returns empty sets without edgeKinds and null for measure beats", () => {
    expect(edgeKindBeatHighlights(projection.edges, { id: "all", label: "Overview" })).toEqual({
      ids: new Set(),
      edgeIds: new Set(),
    });
    expect(edgeKindBeatHighlights(projection.edges, undefined)?.ids.size).toBe(0);
    expect(edgeKindBeatHighlights(undefined, { id: "x", label: "X", filter: { edgeKinds: ["member_of"] } })?.ids.size)
      .toBe(0);
    expect(edgeKindBeatHighlights(projection.edges, { id: "b", label: "B", measure: "single_tie" })).toBeNull();
  });

  it("ignores edges whose kind is missing or unknown to the viewer's projection", () => {
    const edges = [
      { id: "no-kind", from: "p1", to: "p2" },
      { id: "other", kind: "member_of", from: "p1", to: "c1" },
    ];
    const highlights = edgeKindBeatHighlights(edges, { id: "t", label: "T", filter: { edgeKinds: ["connected_to"] } });

    expect(highlights?.ids.size).toBe(0);
    expect(highlights?.edgeIds.size).toBe(0);
  });

  it("intersects kinds and edgeKinds: entities of those kinds on those edges, edges with both ends lit", () => {
    const both: PresentBeat = {
      id: "committee-people",
      label: "People on committees",
      filter: { kinds: ["person"], edgeKinds: ["member_of"] },
    };
    const highlights = beatHighlights(projection, both);

    // p1 and p3 are the people incident to member_of edges; c1 is excluded by kinds,
    // so no member_of edge has both endpoints lit and none stays bright.
    expect([...(highlights?.ids ?? [])].sort()).toEqual(["p1", "p3"]);
    expect(highlights?.edgeIds.size).toBe(0);

    const peopleTies: PresentBeat = {
      id: "ties",
      label: "Ties",
      filter: { kinds: ["person"], edgeKinds: ["connected_to"] },
    };
    const tieHighlights = beatHighlights(projection, peopleTies);
    expect([...(tieHighlights?.ids ?? [])].sort()).toEqual(["p1", "p2"]);
    expect([...(tieHighlights?.edgeIds ?? [])]).toEqual(["e-tie"]);
  });

  it("keeps today's kinds-only and empty-beat behavior through beatHighlights", () => {
    expect([...(beatHighlights(projection, { id: "c", label: "Committees", filter: { kinds: ["committee"] } })?.ids ?? [])])
      .toEqual(["c1"]);
    expect(beatHighlights(projection, { id: "c", label: "Committees", filter: { kinds: ["committee"] } })?.edgeIds.size)
      .toBe(0);
    expect(beatHighlights(projection, { id: "all", label: "Overview" })).toEqual({ ids: new Set(), edgeIds: new Set() });
    expect(beatHighlights(null, { id: "t", label: "T", filter: { edgeKinds: ["connected_to"] } })?.ids.size).toBe(0);
    expect(beatHighlights(projection, { id: "b", label: "B", measure: "betweenness_top_n", topN: 1 })).toBeNull();
  });

  it("feeds the existing focus pipeline: highlighted edges join adjacentEdgeIds, no new render pass", () => {
    const highlights = beatHighlights(projection, { id: "ties", label: "Ties", filter: { edgeKinds: ["connected_to"] } });
    const focus = computeFocusSet(projection, null, highlights?.ids, highlights?.edgeIds);

    expect(focus).not.toBeNull();
    expect([...(focus?.neighborIds ?? [])].sort()).toEqual(["p1", "p2"]);
    expect([...(focus?.adjacentEdgeIds ?? [])]).toEqual(["e-tie"]);
    // Highlighted edges alone are enough to enter the dim/highlight state.
    expect(computeFocusSet(projection, null, new Set(), new Set(["e-tie"]))?.adjacentEdgeIds.has("e-tie")).toBe(true);
    expect(computeFocusSet(projection, null, new Set(), new Set())).toBeNull();
  });

  it("adds a focusEntityId beat's neighborhood to the filter highlights (c: highlight without camera)", () => {
    const highlights = beatHighlights(projection, { id: "one", label: "One node", focusEntityId: "p2", camera: "hold" });
    const focus = computeFocusSet(projection, "p2", highlights?.ids, highlights?.edgeIds);

    expect(focus?.focusedId).toBe("p2");
    expect([...(focus?.neighborIds ?? [])].sort()).toEqual(["o1", "p1"]);
    expect([...(focus?.adjacentEdgeIds ?? [])].sort()).toEqual(["e-aff", "e-tie"]);
  });
});

describe("presenter camera option (D-103.3)", () => {
  const focusBeat: PresentBeat = { id: "one", label: "One node", focusEntityId: "p2" };
  const plainBeat: PresentBeat = { id: "all", label: "Overview" };
  const enterWithFocus: CameraMoveContext = {
    present: true,
    focusedId: "p2",
    focusChanged: true,
    beatChanged: true,
    rebuilt: false,
  };
  const beatOnly: CameraMoveContext = {
    present: true,
    focusedId: null,
    focusChanged: false,
    beatChanged: true,
    rebuilt: false,
  };

  it("keeps today's behavior when camera is undefined: fly to a new focus, otherwise fit", () => {
    expect(beatCameraMove(focusBeat, enterWithFocus)).toBe("fly");
    expect(beatCameraMove(plainBeat, beatOnly)).toBe("fit");
    expect(beatCameraMove(plainBeat, { ...beatOnly, beatChanged: false, rebuilt: true })).toBe("fit");
    // Same focused entity carried across two beats: no new focus, so a fit.
    expect(beatCameraMove(focusBeat, { ...enterWithFocus, focusChanged: false })).toBe("fit");
    expect(beatCameraMove(plainBeat, { ...beatOnly, beatChanged: false })).toBe("none");
  });

  it("hold performs no camera motion at all, even with a focusEntityId", () => {
    const hold: PresentBeat = { ...focusBeat, camera: "hold" };
    expect(beatCameraMove(hold, enterWithFocus)).toBe("none");
    expect(beatCameraMove(hold, { ...enterWithFocus, rebuilt: true })).toBe("none");
    expect(beatCameraMove({ ...plainBeat, camera: "hold" }, beatOnly)).toBe("none");
  });

  it("fit forces a zoom-to-fit and never flies, even with a focusEntityId", () => {
    const fit: PresentBeat = { ...focusBeat, camera: "fit" };
    expect(beatCameraMove(fit, enterWithFocus)).toBe("fit");
    expect(beatCameraMove({ ...plainBeat, camera: "fit" }, beatOnly)).toBe("fit");
    expect(beatCameraMove(fit, { ...enterWithFocus, beatChanged: false, rebuilt: false })).toBe("none");
  });

  it("fly forces the flight whenever a focused entity is set, and fits without one", () => {
    const fly: PresentBeat = { ...focusBeat, camera: "fly" };
    expect(beatCameraMove(fly, enterWithFocus)).toBe("fly");
    expect(beatCameraMove(fly, { ...enterWithFocus, focusChanged: false })).toBe("fly");
    expect(beatCameraMove({ ...plainBeat, camera: "fly" }, beatOnly)).toBe("fit");
  });

  it("ignores the beat outside present mode", () => {
    const outside: CameraMoveContext = {
      present: false,
      focusedId: "p2",
      focusChanged: true,
      beatChanged: false,
      rebuilt: false,
    };
    expect(beatCameraMove({ ...focusBeat, camera: "hold" }, outside)).toBe("fly");
    expect(beatCameraMove(undefined, { ...outside, focusedId: null, focusChanged: false, rebuilt: true })).toBe("none");
  });
});

describe("presenter hotkey rail (CS-06: beatIndexForKey)", () => {
  const beats: PresentBeat[] = [
    { id: "network-overview", label: "Overview" },
    { id: "members", label: "Members", filter: { kinds: ["person"] } },
    { id: "committees", label: "Committees", filter: { kinds: ["committee"] } },
    { id: "organizations", label: "Organizations", filter: { kinds: ["organization"] } },
    { id: "shared-priorities", label: "Shared priorities", filter: { edgeKinds: ["connected_to"] } },
    { id: "one-node", label: "One node", focusEntityId: "p2", camera: "hold" },
    { id: "constellation", label: "Constellation" },
  ];

  it("resolves every hotkey to the index of its beat id, never by position", () => {
    expect(beatIndexForKey("c", beats)).toBe(2);
    expect(beats[beatIndexForKey("c", beats) ?? -1]?.id).toBe("committees");
    expect(beatIndexForKey("o", beats)).toBe(3);
    expect(beats[beatIndexForKey("o", beats) ?? -1]?.id).toBe("organizations");
    expect(beatIndexForKey("m", beats)).toBe(1);
    expect(beats[beatIndexForKey("m", beats) ?? -1]?.id).toBe("members");
    expect(beatIndexForKey("p", beats)).toBe(4);
    expect(beats[beatIndexForKey("p", beats) ?? -1]?.id).toBe("shared-priorities");
    expect(beatIndexForKey("1", beats)).toBe(5);
    expect(beats[beatIndexForKey("1", beats) ?? -1]?.id).toBe("one-node");
    expect(beatIndexForKey("End", beats)).toBe(6);
    expect(beats[beatIndexForKey("End", beats) ?? -1]?.id).toBe("constellation");
  });

  it("is case-insensitive for letter and named keys", () => {
    expect(beatIndexForKey("C", beats)).toBe(2);
    expect(beatIndexForKey("O", beats)).toBe(3);
    expect(beatIndexForKey("M", beats)).toBe(1);
    expect(beatIndexForKey("P", beats)).toBe(4);
    expect(beatIndexForKey("end", beats)).toBe(6);
    expect(beatIndexForKey("END", beats)).toBe(6);
  });

  it("returns null for a key with no mapping", () => {
    expect(beatIndexForKey("a", beats)).toBeNull();
    expect(beatIndexForKey("2", beats)).toBeNull();
    expect(beatIndexForKey("Home", beats)).toBeNull();
    expect(beatIndexForKey(" ", beats)).toBeNull();
  });

  it("returns null (never a fallback index) when the mapped beat id is absent from beats", () => {
    const withoutPriorities = beats.filter((beat) => beat.id !== "shared-priorities");
    expect(beatIndexForKey("p", withoutPriorities)).toBeNull();
    expect(beatIndexForKey("c", [])).toBeNull();
  });
});
