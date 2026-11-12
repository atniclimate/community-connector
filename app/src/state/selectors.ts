import type {
  AppState,
  EntityDetailDto,
  ErrorEnvelopeDto,
  PathDto,
  ProjectionEntityDto,
  RouteCandidateDto,
  SearchState,
  SearchStatus,
  VisibilityMirrorDto,
} from "./state";

/**
 * The viewer's own entity: the first projected entity the core marks
 * `owner_is_viewer`. shortcut: a viewer owning several entities gets the
 * first in projection order; pick by kind when a viewer can own several.
 */
export function selectSelfEntityId(state: AppState): string | null {
  return selectProjectedEntities(state).find((entity) => entity.owner_is_viewer === true)?.id ?? null;
}

export type RoutingCandidateView = RouteCandidateDto & { readonly resting: boolean };

function visiblyResting(entity: ProjectionEntityDto | null): boolean {
  const attributes = entity?.attributes;
  if (typeof attributes !== "object" || attributes === null || Array.isArray(attributes)) {
    return false;
  }
  const capacity = (attributes as Readonly<Record<string, unknown>>).capacity;
  return typeof capacity === "object" && capacity !== null && (capacity as { value?: unknown }).value === "resting";
}

/**
 * Routing candidates in core order, except that a candidate whose projected
 * capacity reads "resting" sorts last so the same few people are not asked
 * first while resting. Presentation order only: it reads what the viewer's
 * projection already shows and never decides visibility (I2).
 */
export function selectRoutingResults(state: AppState): readonly RoutingCandidateView[] {
  const candidates = (state.routing.result ?? []).map((candidate) => ({
    ...candidate,
    resting: visiblyResting(selectProjectedEntityById(state, candidate.entity)),
  }));
  return [...candidates.filter((c) => !c.resting), ...candidates.filter((c) => c.resting)];
}

/** The chosen "who can help" path while its result is the focused entity. */
export function selectChosenRoutePath(state: AppState): PathDto | null {
  const { chosen, result } = state.routing;
  if (chosen === null || state.view.mode !== "focus" || state.view.focusedEntityId !== chosen) {
    return null;
  }
  return result?.find((candidate) => candidate.entity === chosen)?.path ?? null;
}

export type MirrorGapLine = {
  readonly entityId: string;
  /** Null for the record's own presence. */
  readonly attribute: string | null;
  readonly text: string;
};

export type MirrorView = {
  readonly status: SearchStatus;
  readonly error: ErrorEnvelopeDto | null;
  readonly gaps: readonly MirrorGapLine[];
};

function gapText(attribute: string | null, mirror: VisibilityMirrorDto): string {
  const chose = attribute === null ? "You chose" : `For ${attribute}, you chose`;
  return `${chose} ${mirror.visibility}; this record's tier holds it at ${mirror.ceiling}.`;
}

/**
 * Plain-language lines for every value whose chosen circle is wider than
 * its tier allows, from the core's own `tier_capped` and `ceiling` (I2).
 */
export function selectMirrorView(state: AppState): MirrorView {
  const { status, error, result } = state.analysis.mirror;
  const gaps: MirrorGapLine[] = [];
  for (const [entityId, entity] of Object.entries(result?.entities ?? {})) {
    const values: [string | null, VisibilityMirrorDto][] = [[null, entity.presence], ...Object.entries(entity.attributes)];
    for (const [attribute, mirror] of values) {
      if (mirror.tier_capped) {
        gaps.push({ entityId, attribute, text: gapText(attribute, mirror) });
      }
    }
  }
  return { status, error, gaps };
}

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
