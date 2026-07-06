import { describe, expect, it } from "vitest";
import type { Action } from "./actions";
import { reduce } from "./reducer";
import { createInitialState, type AppState, type ProjectionDto } from "./state";
import { createStore } from "./store";

function projection(revision: number): ProjectionDto {
  return {
    revision,
    entities: [{ id: `entity-${revision}`, label: `Entity ${revision}` }],
    edges: [],
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
