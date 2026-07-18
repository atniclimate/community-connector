import type { AppState, EntityDetailDto, ProjectionEntityDto, SearchState } from "./state";

export function selectProjectedEntities(state: AppState): readonly ProjectionEntityDto[] {
  return state.data.projection?.entities ?? [];
}

export function selectProjectedEntityCount(state: AppState): number {
  return selectProjectedEntities(state).length;
}

export function selectProjectedEntityById(
  state: AppState,
  entityId: string,
): ProjectionEntityDto | null {
  return selectProjectedEntities(state).find((entity) => entity.id === entityId) ?? null;
}

export function selectFocusedEntity(state: AppState): ProjectionEntityDto | null {
  if (state.view.mode !== "focus" || state.view.focusedEntityId === null) {
    return null;
  }
  return selectProjectedEntityById(state, state.view.focusedEntityId);
}

export function selectVisibleStoryStep(state: AppState): number | null {
  return state.view.mode === "story" ? state.view.storyStep : null;
}

export function selectSearch(state: AppState): SearchState {
  return state.search;
}

export function selectDetailEntityId(state: AppState): string | null {
  return state.ui.detailEntityId;
}

/**
 * Returns the detail payload only when it matches the requested entity; while
 * a newer detail request is in flight the previous payload is withheld so the
 * panel never renders another entity's data under the wrong heading.
 */
export function selectCurrentDetail(state: AppState): EntityDetailDto | null {
  const detail = state.data.detail;
  if (detail === null || state.ui.detailEntityId === null) {
    return null;
  }
  return detail.id === state.ui.detailEntityId ? detail : null;
}
