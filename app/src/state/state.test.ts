import { describe, expect, it } from "vitest";
import type { Action } from "./actions";
import { reduce } from "./reducer";
import {
  createInitialState,
  type AppState,
  type ProjectionDto,
  type RequestIdentity,
  type ViewerContextDto,
} from "./state";
import { createStore } from "./store";

function projection(revision: number): ProjectionDto {
  return {
    revision,
    entities: [{ id: `entity-${revision}`, label: `Entity ${revision}` }],
    edges: [],
  };
}

function scopedState(
  groupId = "group-alpha",
  viewer: ViewerContextDto = { kind: "anonymous" },
): AppState {
  return reduce(createInitialState(), { kind: "groupLoadRequested", groupId, viewer });
}

function request(state: AppState, requestId: number): RequestIdentity {
  if (state.session.groupId === null) {
    throw new Error("test state has no group");
  }
  return {
    requestId,
    sessionId: state.session.sessionId,
    groupId: state.session.groupId,
    viewer: state.session.viewer,
  };
}

describe("state reducer", () => {
  it("is pure for the same state and action", () => {
    const state = createInitialState();
    const action: Action = { kind: "legendToggled" };

    expect(reduce(state, action)).toEqual(reduce(state, action));
    expect(state).toEqual(createInitialState());
  });

  it("drops stale projections by identity and applies newer revisions", () => {
    const initial = createInitialState();
    const current = reduce(initial, {
      kind: "projectionReceived",
      projection: projection(3),
      revision: 3,
    });

    const stale = reduce(current, {
      kind: "projectionReceived",
      projection: projection(2),
      revision: 2,
    });
    const next = reduce(current, {
      kind: "projectionReceived",
      projection: projection(4),
      revision: 4,
    });

    expect(stale).toBe(current);
    expect(next).not.toBe(current);
    expect(next.session.revision).toBe(4);
  });

  it("accepts the first revision-zero projection", () => {
    const initial = createInitialState();
    const accepted = reduce(initial, {
      kind: "projectionReceived",
      projection: projection(0),
      revision: 0,
    });

    expect(accepted).not.toBe(initial);
    expect(accepted.data.projection?.revision).toBe(0);
  });

  it("keeps mode transitions consistent", () => {
    const focused = reduce(createInitialState(), {
      kind: "entityFocused",
      entityId: "entity-focus",
    });
    const story = reduce(focused, {
      kind: "storyEntered",
      storyId: "story-alpha",
      step: 2,
    });
    const exited = reduce(story, { kind: "storyExited" });

    expect(focused.view).toEqual({
      mode: "focus",
      focusedEntityId: "entity-focus",
      hoveredEntityId: null,
      storyId: null,
      storyStep: 0,
    });
    expect(story.view).toEqual({
      mode: "story",
      focusedEntityId: null,
      hoveredEntityId: null,
      storyId: "story-alpha",
      storyStep: 2,
    });
    expect(exited.view).toEqual({
      mode: "overview",
      focusedEntityId: null,
      hoveredEntityId: null,
      storyId: null,
      storyStep: 0,
    });
  });

  it("stores derived theme results without revision staleness rules", () => {
    const initial = reduce(createInitialState(), {
      kind: "projectionReceived",
      projection: projection(3),
      revision: 3,
    });
    const theme = {
      schema_version: "0.1.0" as const,
      tokens: { "bg.center": { hex: "#0d1017", source: "default" as const } },
    };
    const report = { schema_version: "0.1.0" as const, adjustments: [], warnings: [] };
    const kindMeta = {
      person: { shape: "sphere" as const, label: "Person", colorRole: "kind-1" },
    };
    const themed = reduce(initial, { kind: "themeDerived", theme, report, kindMeta });

    expect(themed.session.revision).toBe(3);
    expect(themed.theme).toEqual({ resolved: theme, report });
    expect(themed.data.kindMeta).toEqual(kindMeta);
  });

  it("tracks the search lifecycle and drops stale query responses", () => {
    const state = scopedState();
    const firstRequest = request(state, 1);
    const requested = reduce(state, {
      kind: "searchRequested",
      query: "kelp",
      request: firstRequest,
    });
    expect(requested.search.status).toBe("pending");
    expect(requested.search.query).toBe("kelp");

    const hit = {
      entity: "00000000-0000-0000-0000-000000000001",
      kind: "person",
      matched_attr: "display_name",
      snippet: "kelp survey",
    };
    const succeeded = reduce(requested, {
      kind: "searchSucceeded",
      query: "kelp",
      hits: [hit],
      request: firstRequest,
    });
    expect(succeeded.search.status).toBe("ready");
    expect(succeeded.search.hits).toEqual([hit]);

    const stale = reduce(succeeded, {
      kind: "searchSucceeded",
      query: "old-query",
      hits: [],
      request: firstRequest,
    });
    expect(stale).toBe(succeeded);

    const staleFailure = reduce(succeeded, {
      kind: "searchFailed",
      query: "old-query",
      error: { code: "ProtocolError", message: "stale" },
      request: firstRequest,
    });
    expect(staleFailure).toBe(succeeded);

    const secondRequest = request(succeeded, 2);
    const pendingFailure = reduce(succeeded, {
      kind: "searchRequested",
      query: "kelp",
      request: secondRequest,
    });
    const failed = reduce(pendingFailure, {
      kind: "searchFailed",
      query: "kelp",
      error: { code: "ProtocolError", message: "boom" },
      request: secondRequest,
    });
    expect(failed.search.status).toBe("error");
    expect(failed.search.error?.message).toBe("boom");
    expect(failed.search.hits).toEqual([]);

    const cleared = reduce(failed, { kind: "searchCleared" });
    expect(cleared.search).toEqual(createInitialState().search);
  });

  it("clears stale search hits on a new projection but keeps the query", () => {
    const state = scopedState();
    const searchRequest = request(state, 1);
    const withHits = reduce(
      reduce(state, { kind: "searchRequested", query: "kelp", request: searchRequest }),
      {
        kind: "searchSucceeded",
        query: "kelp",
        request: searchRequest,
        hits: [
          {
            entity: "00000000-0000-0000-0000-000000000001",
            kind: "person",
            matched_attr: "display_name",
            snippet: "kelp survey",
          },
        ],
      },
    );

    const refreshed = reduce(withHits, {
      kind: "projectionReceived",
      projection: projection(5),
      revision: 5,
    });

    expect(refreshed.search.hits).toEqual([]);
    expect(refreshed.search.status).toBe("idle");
    expect(refreshed.search.query).toBe("kelp");
  });

  it("resets search state on a new group load", () => {
    const state = scopedState();
    const withQuery = reduce(state, {
      kind: "searchRequested",
      query: "kelp",
      request: request(state, 1),
    });
    const reloaded = reduce(withQuery, {
      kind: "groupLoadRequested",
      groupId: "group-beta",
      viewer: { kind: "anonymous" },
    });

    expect(reloaded.search).toEqual(createInitialState().search);
  });

  it("keeps B focused when A detail completes late", () => {
    const state = scopedState();
    const focusedA = reduce(state, { kind: "entityFocused", entityId: "entity-a" });
    const requestA = request(focusedA, 1);
    const pendingA = reduce(focusedA, {
      kind: "detailRequested",
      entityId: "entity-a",
      request: requestA,
    });
    const focusedB = reduce(pendingA, { kind: "entityFocused", entityId: "entity-b" });

    const lateA = reduce(focusedB, {
      kind: "detailReceived",
      entityId: "entity-a",
      detail: { id: "entity-a" },
      request: requestA,
    });

    expect(lateA).toBe(focusedB);
    expect(lateA.ui.detailEntityId).toBe("entity-b");
  });

  it("rejects old-group detail after a new group session starts", () => {
    const oldState = scopedState("group-old");
    const focused = reduce(oldState, { kind: "entityFocused", entityId: "entity-a" });
    const oldRequest = request(focused, 1);
    const pending = reduce(focused, {
      kind: "detailRequested",
      entityId: "entity-a",
      request: oldRequest,
    });
    const newState = reduce(pending, {
      kind: "groupLoadRequested",
      groupId: "group-new",
      viewer: { kind: "anonymous" },
    });

    expect(
      reduce(newState, {
        kind: "detailReceived",
        entityId: "entity-a",
        detail: { id: "entity-a" },
        request: oldRequest,
      }),
    ).toBe(newState);
  });

  it("rejects a repeated query response across viewer and group changes", () => {
    const oldState = scopedState("group-old", { kind: "anonymous" });
    const oldRequest = request(oldState, 1);
    const oldPending = reduce(oldState, {
      kind: "searchRequested",
      query: "kelp",
      request: oldRequest,
    });
    const newState = reduce(oldPending, {
      kind: "groupLoadRequested",
      groupId: "group-new",
      viewer: { kind: "person", person: "00000000-0000-0000-0000-000000000001" },
    });
    const newRequest = request(newState, 2);
    const newPending = reduce(newState, {
      kind: "searchRequested",
      query: "kelp",
      request: newRequest,
    });

    const late = reduce(newPending, {
      kind: "searchSucceeded",
      query: "kelp",
      hits: [],
      request: oldRequest,
    });
    expect(late).toBe(newPending);
    expect(late.search.request?.requestId).toBe(2);
  });

  it("accepts only the newest out-of-order success or failure", () => {
    const state = scopedState();
    const first = request(state, 1);
    const firstPending = reduce(state, { kind: "searchRequested", query: "kelp", request: first });
    const second = request(firstPending, 2);
    const secondPending = reduce(firstPending, {
      kind: "searchRequested",
      query: "kelp",
      request: second,
    });
    const staleFailure = reduce(secondPending, {
      kind: "searchFailed",
      query: "kelp",
      request: first,
      error: { code: "ProtocolError", message: "old failure" },
    });
    expect(staleFailure).toBe(secondPending);

    const currentSuccess = reduce(staleFailure, {
      kind: "searchSucceeded",
      query: "kelp",
      request: second,
      hits: [],
    });
    expect(currentSuccess.search.status).toBe("ready");
  });

  it("stores hovered entity and quality tier through explicit actions", () => {
    const hovered = reduce(createInitialState(), {
      kind: "entityHovered",
      entityId: "entity-hover",
    });
    const tiered = reduce(hovered, { kind: "qualityTierChanged", tier: "C" });

    expect(hovered.view.hoveredEntityId).toBe("entity-hover");
    expect(tiered.ui.qualityTier).toBe("C");
  });
});

describe("store", () => {
  it("notifies once per dispatch and keeps unchanged state identity", () => {
    const unchangedReducer = (state: AppState): AppState => state;
    const store = createStore(createInitialState(), {
      reducer: unchangedReducer,
      devFreeze: false,
    });
    let notifications = 0;
    const before = store.getState();

    store.subscribe(() => {
      notifications += 1;
    });
    store.dispatch({ kind: "legendToggled" });

    expect(notifications).toBe(1);
    expect(store.getState()).toBe(before);
  });

  it("freezes dispatched state snapshots in dev mode", () => {
    const store = createStore(createInitialState(), { devFreeze: true });
    store.dispatch({ kind: "sidebarToggled" });
    const snapshot = store.getState();

    expect(() => {
      (snapshot.session as { loadState: string }).loadState = "error";
    }).toThrow(TypeError);
  });
});
