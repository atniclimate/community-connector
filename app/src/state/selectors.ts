import type { AppState, ProjectionEntityDto } from "./state";

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
