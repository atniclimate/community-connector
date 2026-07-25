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

export type LoadState = "idle" | "loading" | "ready" | "error";
export type ViewMode = "overview" | "focus" | "story";
export type QualityTierName = "A" | "B" | "C" | "D";
export type ShapeName = "sphere" | "cube" | "octahedron" | "tetrahedron" | "torus" | "cone";

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
    intake: initialIntakeState(),
    theme: {
      resolved: null,
      report: null,
    },
  };
}
