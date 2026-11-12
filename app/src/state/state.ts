import type { Theme, ThemeReport } from "../theme/tokens";

export type JsonPrimitive = string | number | boolean | null;
export type JsonValue = JsonPrimitive | JsonObject | readonly JsonValue[];

export type JsonObject = {
  readonly [key: string]: JsonValue;
};

export type ViewerContextDto = JsonObject & {
  readonly kind: string;
};

export type ErrorEnvelopeDto = {
  readonly code: string;
  readonly message: string;
  readonly details?: JsonValue;
};

export type ProjectionEntityDto = {
  readonly id: string;
  readonly kind?: string;
  readonly label?: string;
  readonly [key: string]: JsonValue | undefined;
};

export type ProjectionEdgeDto = {
  readonly id: string;
  readonly from?: string;
  readonly to?: string;
  readonly source?: string;
  readonly target?: string;
  readonly [key: string]: JsonValue | undefined;
};

export type ProjectionDto = JsonObject & {
  readonly group_id?: string;
  readonly groupId?: string;
  readonly viewer_fingerprint?: string;
  readonly viewerFingerprint?: string;
  readonly revision?: number;
  readonly entities?: readonly ProjectionEntityDto[];
  readonly edges?: readonly ProjectionEdgeDto[];
};

export type EntityDetailDto = JsonObject & {
  readonly id: string;
  readonly kind?: string;
  readonly owner_is_viewer?: boolean;
  readonly tier?: string;
  readonly provenance?: JsonObject;
  readonly attributes?: JsonObject;
};

export type SearchHitDto = {
  readonly entity: string;
  readonly kind: string;
  readonly matched_attr: string;
  readonly snippet: string;
};

export type SearchStatus = "idle" | "pending" | "ready" | "error";

export type RequestIdentity = {
  readonly requestId: number;
  readonly sessionId: number;
  readonly groupId: string;
  readonly viewer: ViewerContextDto;
};

export type SearchState = {
  readonly query: string;
  readonly status: SearchStatus;
  readonly hits: readonly SearchHitDto[];
  readonly error: ErrorEnvelopeDto | null;
  readonly request: RequestIdentity | null;
};

/** One status/result/error/request record for a viewer-scoped core call;
 * results are dropped on staleness exactly like search (ADR-003 D3). */
export type RequestSlice<T> = {
  readonly status: SearchStatus;
  readonly result: T | null;
  readonly error: ErrorEnvelopeDto | null;
  readonly request: RequestIdentity | null;
};

/** cn-graph `Path` over the viewer projection; null when no route exists. */
export type PathDto = {
  readonly nodes: readonly string[];
  readonly edges: readonly string[];
  readonly cost: number;
};

/** A "who can help" candidate: the search hits that matched it and the
 * route from the viewer's own entity, or null when none is within reach. */
export type RouteCandidateDto = {
  readonly entity: string;
  readonly hits: readonly SearchHitDto[];
  readonly path: PathDto | null;
};

export type RoutingState = RequestSlice<readonly RouteCandidateDto[]> & {
  readonly query: string;
};

export type NeighborhoodDto = {
  readonly center: string;
  readonly layers: readonly (readonly string[])[];
};

export type ArticulationPointDto = {
  readonly entity: string;
  readonly parts: number;
  readonly part_sizes: readonly number[];
  readonly explanation: string;
};

export type BridgeTieDto = {
  readonly edge: string;
  readonly from: string;
  readonly to: string;
  readonly from_side: number;
  readonly to_side: number;
  readonly explanation: string;
};

export type ContinuanceDto = {
  readonly articulation_points: Readonly<Record<string, ArticulationPointDto>>;
  readonly bridges: Readonly<Record<string, BridgeTieDto>>;
  readonly components: Readonly<Record<string, number>>;
};

export type IntroductionDto = {
  readonly candidate: string;
  readonly score: number;
  readonly introducers: readonly { readonly entity: string; readonly willing: boolean }[];
  readonly shared_tags: {
    readonly center_seeks: readonly string[];
    readonly candidate_seeks: readonly string[];
  };
  readonly explanation: string;
};

/** cn-perm `VisibilityMirror`: the chosen circle, the tier ceiling, and
 * what each audience receives - all computed by the core (I2). */
export type VisibilityMirrorDto = {
  readonly visibility: string;
  readonly effective_tier: string;
  readonly ceiling: string;
  readonly audiences: Readonly<Record<string, boolean>>;
  readonly tier_capped: boolean;
  readonly ceiling_blocks: readonly string[];
};

export type EntityMirrorDto = {
  readonly kind: string;
  readonly presence: VisibilityMirrorDto;
  readonly attributes: Readonly<Record<string, VisibilityMirrorDto>>;
};

export type RecordMirrorDto = {
  readonly owner: string | null;
  readonly entities: Readonly<Record<string, EntityMirrorDto>>;
};

export type AnalysisResults = {
  readonly neighborhood: NeighborhoodDto;
  readonly continuance: ContinuanceDto;
  readonly introductions: readonly IntroductionDto[];
  readonly mirror: RecordMirrorDto;
};

export type AnalysisName = keyof AnalysisResults;

export type AnalysisState = {
  readonly [K in AnalysisName]: RequestSlice<AnalysisResults[K]>;
};

export type LoadState = "idle" | "loading" | "ready" | "error";
export type ViewMode = "overview" | "focus" | "story" | "present";
export type QualityTierName = "A" | "B" | "C" | "D";
export type ShapeName = "sphere" | "cube" | "octahedron" | "tetrahedron" | "torus" | "cone";

export type PresentMeasure = "betweenness_top_n" | "single_tie";

/**
 * Camera behavior on a beat change. Undefined keeps the pre-D-103 behavior
 * (fly to a newly focused entity, otherwise fit the beat's positions).
 * "hold" performs no camera motion at all, even with a focusEntityId, so an
 * opacity-only spotlight satisfies the stage's "no fly-ins" rule (D-103.3).
 * "fit" always frames the beat's positions; "fly" always flies to the
 * focused entity when one is set (and fits when none is).
 */
export type PresentCamera = "hold" | "fit" | "fly";

export type PresentBeat = {
  readonly id: string;
  readonly label: string;
  readonly focusEntityId?: string;
  readonly camera?: PresentCamera;
  readonly filter?: {
    readonly kinds?: readonly string[];
    /** Entities incident to an edge of one of these kinds are highlighted and
     * those edges stay bright while others dim. With `kinds` also set the
     * entity set is the intersection (see viz/presenter.ts beatHighlights). */
    readonly edgeKinds?: readonly string[];
  };
  readonly measure?: PresentMeasure;
  readonly topN?: number;
  /**
   * Present mode only: the entity kinds whose labels may render on this
   * beat. When set, every other label is hidden - including the focused
   * entity and its neighbors - so a beat that lights people can name their
   * committees without ever naming a person (D-099 stage name gate). The
   * gate fails closed: an empty array or an undefined field labels nothing
   * in present mode, and "person" is always excluded even if listed (a
   * sheet naming it is rejected at load). Ignored outside present mode.
   */
  readonly labelKinds?: readonly string[];
};

export type PresentationState = {
  readonly beats: readonly PresentBeat[];
  readonly beatIndex: number;
  readonly loadState: LoadState;
  /**
   * Whether the operator's button rail is shown in present mode (I4: UI
   * state lives here, not in the renderer). Toggled by `presentRailToggled`
   * (`r`); reset to false whenever the view mode enters or leaves "present",
   * so the stage never shows the rail unless the operator asks. The renderer
   * only projects it onto the rail's `hidden` attribute.
   */
  readonly railShown: boolean;
};

export type KindMeta = {
  readonly shape: ShapeName;
  readonly label: string;
  readonly colorRole: string;
};

/** One queue record as summarized for the wizard (display-only; the
 * native CLI re-verifies everything authoritatively). */
export type IntakeRecordSummaryDto = {
  readonly recordId: string;
  /** Sidecar review_state; null when the sidecar is missing/unreadable
   * (the wizard refuses to stage decisions for such records). */
  readonly reviewState: string | null;
  readonly decisionGeneration: number;
  /** The record_checksum value - the decision message's payload binding. */
  readonly payloadDigest: string;
  /** The payload_hash value - the semantic dedup key component. */
  readonly payloadHash: string;
  readonly stagedAtMs: number;
  readonly payload: JsonObject;
  /** The full queue record as read, for the review view's core calls. */
  readonly record: JsonObject;
};

export type IntakeStatus = "idle" | "working" | "ready" | "error";

export type IntakeState = {
  /** Granted FSA queue-directory name; null until the facilitator grants.
   * The live handle itself is infrastructure owned by the state module's
   * effects layer (round-1 F4), never store data. */
  readonly dirName: string | null;
  readonly status: IntakeStatus;
  readonly records: readonly IntakeRecordSummaryDto[];
  /** Unconsumed decision files staged and awaiting `cn intake apply`. */
  readonly pendingDecisionFiles: number;
  /** The record open in the review view (round-1 F4: UI selection lives
   * in the store, not in a component). */
  readonly reviewRecordId: string | null;
  /** Per-file scan problems surfaced for I12 visibility (round-1 F3:
   * "record unreadable" is never silently identical to "no record"). */
  readonly scanIssues: readonly string[];
  /** Durable operator notices (round-3 F3): never cleared by scans -
   * only by a directory change - so persistence warnings survive into
   * painted state. */
  readonly notices: readonly string[];
  readonly lastError: ErrorEnvelopeDto | null;
};

export function initialIntakeState(): IntakeState {
  return {
    dirName: null,
    status: "idle",
    records: [],
    pendingDecisionFiles: 0,
    reviewRecordId: null,
    scanIssues: [],
    notices: [],
    lastError: null,
  };
}

export interface AppState {
  readonly session: {
    readonly groupId: string | null;
    readonly viewer: ViewerContextDto;
    readonly sessionId: number;
    readonly revision: number;
    readonly loadState: LoadState;
    readonly lastError: ErrorEnvelopeDto | null;
  };
  readonly view: {
    readonly mode: ViewMode;
    readonly focusedEntityId: string | null;
    readonly hoveredEntityId: string | null;
    readonly storyId: string | null;
    readonly storyStep: number;
  };
  readonly ui: {
    readonly legendOpen: boolean;
    readonly sidebarOpen: boolean;
    readonly detailEntityId: string | null;
    readonly reducedMotion: boolean;
    readonly qualityTier: QualityTierName;
  };
  readonly data: {
    readonly projection: ProjectionDto | null;
    readonly detail: EntityDetailDto | null;
    readonly detailRequest: (RequestIdentity & { readonly entityId: string }) | null;
    readonly detailError: ErrorEnvelopeDto | null;
    readonly kindMeta: Readonly<Record<string, KindMeta>>;
  };
  readonly search: SearchState;
  /** "Who can help" routing: search plus paths from the viewer's own entity. */
  readonly routing: RoutingState;
  /** Person-centered core calls; reset with search on group load or projection. */
  readonly analysis: AnalysisState;
  readonly presentation: PresentationState;
  /** Intake queue slice: survives group reloads (the queue is the
   * facilitator's ops directory, not group-session data). */
  readonly intake: IntakeState;
  readonly theme: {
    readonly resolved: Theme | null;
    readonly report: ThemeReport | null;
  };
}

export function initialSearchState(): SearchState {
  return {
    query: "",
    status: "idle",
    hits: [],
    error: null,
    request: null,
  };
}

function idleSlice<T>(): RequestSlice<T> {
  return { status: "idle", result: null, error: null, request: null };
}

export function initialRoutingState(query = ""): RoutingState {
  return { ...idleSlice<readonly RouteCandidateDto[]>(), query };
}

export function initialAnalysisState(): AnalysisState {
  return {
    neighborhood: idleSlice(),
    continuance: idleSlice(),
    introductions: idleSlice(),
    mirror: idleSlice(),
  };
}

export function initialPresentationState(): PresentationState {
  return {
    beats: [],
    beatIndex: 0,
    loadState: "idle",
    railShown: false,
  };
}

export function createInitialState(reducedMotion = false): AppState {
  return {
    session: {
      groupId: null,
      viewer: { kind: "anonymous" },
      sessionId: 0,
      revision: 0,
      loadState: "idle",
      lastError: null,
    },
    view: {
      mode: "overview",
      focusedEntityId: null,
      hoveredEntityId: null,
      storyId: null,
      storyStep: 0,
    },
    ui: {
      legendOpen: false,
      sidebarOpen: true,
      detailEntityId: null,
      reducedMotion,
      qualityTier: "B",
    },
    data: {
      projection: null,
      detail: null,
      detailRequest: null,
      detailError: null,
      kindMeta: {},
    },
    search: initialSearchState(),
    routing: initialRoutingState(),
    analysis: initialAnalysisState(),
    presentation: initialPresentationState(),
    intake: initialIntakeState(),
    theme: {
      resolved: null,
      report: null,
    },
  };
}
