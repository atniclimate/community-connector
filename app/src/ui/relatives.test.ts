import { describe, expect, it } from "vitest";
import { reduce } from "../state/reducer";
import { selectChosenRoutePath } from "../state/selectors";
import { createInitialState, type AppState, type ContinuanceDto, type IntroductionDto, type ProjectionDto, type RecordMirrorDto } from "../state/state";
import {
  AUDIENCES,
  UNNAMED_RELATIVE,
  chainText,
  helperViews,
  introductionViews,
  mirrorRows,
  relativesContext,
  restingOnYouLines,
  sliceNotice,
} from "./relatives";

// Synthetic entities only (I1).
const name = (value: string) => ({ type: "text", value });
const willing = (value: string) => ({ type: "enum", value });

const PROJECTION: ProjectionDto = {
  revision: 1,
  entities: [
    { id: "self", kind: "person", owner_is_viewer: true, attributes: { display_name: name("Self Example") } },
    { id: "ana", kind: "person", attributes: { display_name: name("Ana Example"), willing_to_introduce: willing("yes") } },
    { id: "ben", kind: "person", attributes: { display_name: name("Ben Example"), willing_to_introduce: willing("no") } },
    // Willingness hidden from this viewer: must read exactly like "no".
    { id: "cai", kind: "person", attributes: { display_name: name("Cai Example") } },
    {
      id: "dee",
      kind: "person",
      attributes: { display_name: name("Dee Example"), offering: { type: "tags", value: ["water", "grants"] } },
    },
  ],
  edges: [],
};

function loaded(): AppState {
  const scoped = reduce(createInitialState(), {
    kind: "groupLoadRequested",
    groupId: "group-alpha",
    viewer: { kind: "person", person: "p-self" },
  });
  return reduce(scoped, { kind: "projectionReceived", projection: PROJECTION, revision: 1 });
}

const ctx = relativesContext(loaded());

describe("chainText introducer naming", () => {
  it("names only a visibly willing relative and keeps every other step anonymous", () => {
    const path = { nodes: ["self", "ana", "ben", "cai", "dee"], edges: ["e1", "e2", "e3", "e4"], cost: 4 };
    expect(chainText(ctx, path)).toBe(
      `From you through Ana Example (willing to introduce), then through ${UNNAMED_RELATIVE}, then through ${UNNAMED_RELATIVE}.`,
    );
  });

  it("says when there is no chain or a direct tie", () => {
    expect(chainText(ctx, null)).toBe("No chain of relatives within 4 steps reaches them.");
    expect(chainText(ctx, { nodes: ["self", "dee"], edges: ["e1"], cost: 1 })).toBe("You are already directly connected.");
  });
});

describe("helperViews", () => {
  it("carries the offering tags, the chain, and the resting flag", () => {
    const views = helperViews(ctx, [{ entity: "dee", hits: [], path: null, resting: true }]);
    expect(views).toEqual([
      {
        entity: "dee",
        label: "Dee Example",
        offering: "water, grants",
        chain: "No chain of relatives within 4 steps reaches them.",
        resting: true,
      },
    ]);
  });
});

describe("introductionViews", () => {
  const suggestion = (candidate: string): IntroductionDto => ({
    candidate,
    score: 1,
    introducers: [
      { entity: "ana", willing: true },
      { entity: "ben", willing: false },
      { entity: "cai", willing: false },
    ],
    shared_tags: { center_seeks: ["water"], candidate_seeks: [] },
    explanation: "core text",
  });

  it("keeps the top five in core order with the same naming rule and shared tags", () => {
    const views = introductionViews(ctx, ["dee", "ben", "cai", "ana", "self", "dee"].map(suggestion));
    expect(views).toHaveLength(5);
    expect(views.map((view) => view.entity)).toEqual(["dee", "ben", "cai", "ana", "self"]);
    expect(views[0]).toEqual({
      entity: "dee",
      label: "Dee Example",
      reason: "You share 3 mutual relatives: Ana Example (willing to introduce) and 2 relatives in this group.",
      shared: ["They offer what you seek: water"],
    });
  });
});

describe("restingOnYouLines", () => {
  it("writes plain sentences in core order without ranking", () => {
    const continuance: ContinuanceDto = {
      articulation_points: {
        self: { entity: "self", parts: 2, part_sizes: [1, 4], explanation: "x" },
      },
      bridges: {
        e9: { edge: "e9", from: "ana", to: "self", from_side: 3, to_side: 2, explanation: "x" },
      },
      components: {},
    };
    expect(restingOnYouLines(ctx, continuance)).toEqual([
      "You are the only tie between 2 parts of this group, of 1, 4 people.",
      "Your tie with Ana Example is the only link between a group of 2 and a group of 3.",
    ]);
  });
});

describe("mirrorRows", () => {
  it("has one row per value in Public, Network, Group, Trusted order", () => {
    const audiences = { public: false, network: false, group: true, trusted: true };
    const value = { visibility: "group", effective_tier: "T2", ceiling: "group", audiences, tier_capped: false, ceiling_blocks: [] };
    const mirror: RecordMirrorDto = {
      owner: "p-self",
      entities: { self: { kind: "person", presence: value, attributes: { willing_to_introduce: value } } },
    };
    expect(AUDIENCES.map(([key]) => key)).toEqual(["public", "network", "group", "trusted"]);
    expect(mirrorRows(ctx, mirror)).toEqual([
      { label: "the record itself", receives: [false, false, true, true] },
      { label: "willing to introduce", receives: [false, false, true, true] },
    ]);
  });
});

describe("sliceNotice", () => {
  it("surfaces pending and error states instead of an empty section (I3)", () => {
    const idle = { status: "idle", result: null, error: null, request: null } as const;
    expect(sliceNotice(idle, "x")).toBeNull();
    expect(sliceNotice({ ...idle, status: "pending" }, "x")).toBe("Loading x.");
    expect(sliceNotice({ ...idle, status: "error", error: { code: "ProtocolError", message: "bad" } }, "x")).toBe(
      "Could not load x: bad",
    );
  });
});

describe("routeResultChosen", () => {
  it("focuses the result and exposes its path only while it stays focused", () => {
    const path = { nodes: ["self", "ana", "dee"], edges: ["e1", "e2"], cost: 2 };
    let state = loaded();
    const request = { requestId: 7, sessionId: state.session.sessionId, groupId: "group-alpha", viewer: state.session.viewer };
    state = reduce(state, { kind: "routeRequested", query: "water", request });
    state = reduce(state, { kind: "routeSucceeded", query: "water", request, candidates: [{ entity: "dee", hits: [], path }] });
    expect(selectChosenRoutePath(state)).toBeNull();
    state = reduce(state, { kind: "routeResultChosen", entityId: "dee" });
    expect(state.view).toMatchObject({ mode: "focus", focusedEntityId: "dee" });
    expect(selectChosenRoutePath(state)).toEqual(path);
    state = reduce(state, { kind: "entityFocused", entityId: "ana" });
    expect(selectChosenRoutePath(state)).toBeNull();
  });
});
