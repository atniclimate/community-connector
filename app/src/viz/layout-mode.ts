import { selectEgoNeighborhood } from "../state/selectors";
import type { AppState, NeighborhoodDto, ProjectionEntityDto } from "../state/state";
import { computeEgoLayout, computeLayout, type LayoutResult } from "./layout";

/**
 * Layout-mode wiring for the render loop, kept out of index.ts (I5).
 * The source is compared by identity in needsRebuild: a mode switch or a new
 * neighborhood result rebuilds the layers. A switch swaps positions in one
 * rebuild with no tween, so nothing animates, under reduced motion or not.
 */
export type LayoutSource = NeighborhoodDto | null;

/** The neighborhood the ring layout is built from, or null for the whole network. */
export function layoutSource(state: AppState): LayoutSource {
  return state.ui.layoutMode === "mine" ? selectEgoNeighborhood(state) : null;
}

export function layoutFor(entities: readonly ProjectionEntityDto[], source: LayoutSource): LayoutResult {
  return source === null ? computeLayout(entities) : computeEgoLayout(entities, source.center, source.layers);
}
