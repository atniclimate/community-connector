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
};

export type LoadState = "idle" | "loading" | "ready" | "error";
export type ViewMode = "overview" | "focus" | "story";

export interface AppState {
  readonly session: {
    readonly groupId: string | null;
    readonly viewer: ViewerContextDto;
    readonly revision: number;
    readonly loadState: LoadState;
    readonly lastError: ErrorEnvelopeDto | null;
  };
  readonly view: {
    readonly mode: ViewMode;
    readonly focusedEntityId: string | null;
    readonly storyId: string | null;
    readonly storyStep: number;
  };
  readonly ui: {
    readonly legendOpen: boolean;
    readonly sidebarOpen: boolean;
    readonly detailEntityId: string | null;
    readonly reducedMotion: boolean;
  };
  readonly data: {
    readonly projection: ProjectionDto | null;
    readonly detail: EntityDetailDto | null;
  };
}

export function createInitialState(reducedMotion = false): AppState {
  return {
    session: {
      groupId: null,
      viewer: { kind: "anonymous" },
      revision: 0,
      loadState: "idle",
      lastError: null,
    },
    view: {
      mode: "overview",
      focusedEntityId: null,
      storyId: null,
      storyStep: 0,
    },
    ui: {
      legendOpen: false,
      sidebarOpen: true,
      detailEntityId: null,
      reducedMotion,
    },
    data: {
      projection: null,
      detail: null,
    },
  };
}
