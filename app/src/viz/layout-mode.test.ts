import { describe, expect, it } from "vitest";
import { reduce } from "../state/reducer";
import { createInitialState, type ProjectionEntityDto } from "../state/state";
import { RENDER_TOKENS } from "./config";
import { computeEgoLayout, layoutSnapshot } from "./layout";
import { layoutFor, layoutSource } from "./layout-mode";

// Synthetic ids only (I1).
const ENTITIES: readonly ProjectionEntityDto[] = ["self", "a1", "a2", "b1", "b2", "b3", "far1", "far2"].map((id) => ({
  id,
  kind: "person",
  attributes: {},
}));
const LAYERS = [["a1", "a2"], ["b1", "b2", "b3"]];

function planarRadius(layout: ReturnType<typeof computeEgoLayout>, id: string): number {
  const position = layout.positions.get(id);
  return position === undefined ? Number.NaN : Math.hypot(position.x, position.y);
}

describe("computeEgoLayout", () => {
  const spacing = RENDER_TOKENS.layout.radius / (LAYERS.length + 1);

  it("places the center at the origin", () => {
    const position = computeEgoLayout(ENTITIES, "self", LAYERS).positions.get("self");
    expect(position?.toArray()).toEqual([0, 0, 0]);
  });

  it("puts hop k on ring k and unreached entities on one outer ring", () => {
    const layout = computeEgoLayout(ENTITIES, "self", LAYERS);
    for (const id of ["a1", "a2"]) expect(planarRadius(layout, id)).toBeCloseTo(spacing);
    for (const id of ["b1", "b2", "b3"]) expect(planarRadius(layout, id)).toBeCloseTo(2 * spacing);
    for (const id of ["far1", "far2"]) expect(planarRadius(layout, id)).toBeCloseTo(3 * spacing);
    expect(layout.positions.size).toBe(ENTITIES.length);
  });

  it("is deterministic and independent of entity order", () => {
    const forward = layoutSnapshot(computeEgoLayout(ENTITIES, "self", LAYERS));
    expect(layoutSnapshot(computeEgoLayout(ENTITIES, "self", LAYERS))).toEqual(forward);
    expect(layoutSnapshot(computeEgoLayout([...ENTITIES].reverse(), "self", LAYERS))).toEqual(forward);
  });

  it("places only projected entities, so a layer id with no entity leaves no trace", () => {
    const layout = computeEgoLayout(ENTITIES, "self", [["a1", "a2", "hidden"], ...LAYERS.slice(1)]);
    expect(layout.positions.has("hidden")).toBe(false);
    expect(layoutSnapshot(layout)).toEqual(layoutSnapshot(computeEgoLayout(ENTITIES, "self", LAYERS)));
  });
});

describe("layout mode", () => {
  it("switches through layoutModeChanged and keeps state on a repeat", () => {
    const initial = createInitialState();
    expect(initial.ui.layoutMode).toBe("whole");
    const mine = reduce(initial, { kind: "layoutModeChanged", mode: "mine" });
    expect(mine.ui.layoutMode).toBe("mine");
    expect(reduce(mine, { kind: "layoutModeChanged", mode: "mine" })).toBe(mine);
    expect(reduce(mine, { kind: "layoutModeChanged", mode: "whole" }).ui.layoutMode).toBe("whole");
  });

  it("uses the whole-network layout until the viewer's neighborhood is ready", () => {
    const state = reduce(createInitialState(), { kind: "layoutModeChanged", mode: "mine" });
    expect(layoutSource(state)).toBeNull();
    const whole = layoutFor(ENTITIES, null);
    expect(whole.positions.get("self")?.toArray()).not.toEqual([0, 0, 0]);
  });
});
