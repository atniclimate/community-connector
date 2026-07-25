import type {
  EntityDetailDto,
  ErrorEnvelopeDto,
  JsonObject,
  ProjectionDto,
  SearchHitDto,
  ViewerContextDto,
} from "../state/state";

export type WorkerRequest =
  | {
      readonly correlationId: number;
      readonly kind: "loadGroup";
      readonly groupId: string;
      readonly viewer: ViewerContextDto;
      readonly templateJson: string;
      readonly opsJsonl: string;
    }
  | {
      readonly correlationId: number;
      readonly kind: "projection";
      readonly groupId: string;
      readonly viewer: ViewerContextDto;
    }
  | {
      readonly correlationId: number;
      readonly kind: "entityDetail";
      readonly groupId: string;
      readonly viewer: ViewerContextDto;
      readonly entityId: string;
    }
  | {
      readonly correlationId: number;
      readonly kind: "submitOps";
      readonly groupId: string;
      readonly viewer: ViewerContextDto;
      readonly opsJson: string;
    }
  | {
      readonly correlationId: number;
      readonly kind: "search";
      readonly groupId: string;
      readonly viewer: ViewerContextDto;
      readonly query: JsonObject;
    }
  | {
      readonly correlationId: number;
      readonly kind: "queryPaths";
      readonly groupId: string;
      readonly viewer: ViewerContextDto;
      readonly request: JsonObject;
    }
  | {
      readonly correlationId: number;
      readonly kind: "queryNeighborhood";
      readonly groupId: string;
      readonly viewer: ViewerContextDto;
      readonly request: JsonObject;
    }
  | {
      readonly correlationId: number;
      readonly kind: "viewerRoles";
      readonly groupId: string;
      readonly viewer: ViewerContextDto;
    }
  | {
      readonly correlationId: number;
      readonly kind: "intakeStageRecord";
      readonly payloadJson: string;
      readonly stagedAtMs: number;
    }
  | {
      readonly correlationId: number;
      readonly kind: "intakeBuildDecision";
      readonly requestJson: string;
    }
  | {
      readonly correlationId: number;
      readonly kind: "intakeValidateRecord";
      readonly groupId: string;
      readonly recordJson: string;
    }
  | {
      readonly correlationId: number;
      readonly kind: "intakeDedupCheck";
      readonly recordJson: string;
      readonly existingKeysJson: string;
    }
  | {
      readonly correlationId: number;
      readonly kind: "intakeNearDuplicates";
      readonly groupId: string;
      readonly viewer: ViewerContextDto;
      readonly requestJson: string;
    };

export type WorkerRequestInput =
  | Omit<Extract<WorkerRequest, { readonly kind: "loadGroup" }>, "correlationId">
  | Omit<Extract<WorkerRequest, { readonly kind: "projection" }>, "correlationId">
  | Omit<Extract<WorkerRequest, { readonly kind: "entityDetail" }>, "correlationId">
  | Omit<Extract<WorkerRequest, { readonly kind: "submitOps" }>, "correlationId">
  | Omit<Extract<WorkerRequest, { readonly kind: "search" }>, "correlationId">
  | Omit<Extract<WorkerRequest, { readonly kind: "queryPaths" }>, "correlationId">
  | Omit<Extract<WorkerRequest, { readonly kind: "queryNeighborhood" }>, "correlationId">
  | Omit<Extract<WorkerRequest, { readonly kind: "viewerRoles" }>, "correlationId">
  | Omit<Extract<WorkerRequest, { readonly kind: "intakeStageRecord" }>, "correlationId">
  | Omit<Extract<WorkerRequest, { readonly kind: "intakeBuildDecision" }>, "correlationId">
  | Omit<Extract<WorkerRequest, { readonly kind: "intakeValidateRecord" }>, "correlationId">
  | Omit<Extract<WorkerRequest, { readonly kind: "intakeDedupCheck" }>, "correlationId">
  | Omit<Extract<WorkerRequest, { readonly kind: "intakeNearDuplicates" }>, "correlationId">;

export type WorkerResponse =
  | {
      readonly correlationId: number;
      readonly ok:
        | ProjectionDto
        | EntityDetailDto
        | readonly SearchHitDto[]
        | readonly JsonObject[]
        | JsonObject;
    }
  | {
      readonly correlationId: number;
      readonly err: ErrorEnvelopeDto;
    };

export type Envelope<T> =
  | {
      readonly ok: T;
    }
  | {
      readonly err: ErrorEnvelopeDto;
    };

export function protocolError(message: string): ErrorEnvelopeDto {
  return {
    code: "ProtocolError",
    message,
  };
}
