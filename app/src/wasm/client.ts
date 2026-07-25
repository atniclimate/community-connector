import type {
  EntityDetailDto,
  ErrorEnvelopeDto,
  JsonObject,
  ProjectionDto,
  SearchHitDto,
  ViewerContextDto,
} from "../state/state";
import type { WorkerRequest, WorkerRequestInput, WorkerResponse } from "./protocol";
import { protocolError } from "./protocol";

export type WorkerMessageEvent = {
  readonly data: WorkerResponse;
};

export type WasmTransport = {
  readonly postMessage: (request: WorkerRequest) => void;
  readonly addEventListener: (
    type: "message",
    listener: (event: WorkerMessageEvent) => void,
  ) => void;
  readonly removeEventListener: (
    type: "message",
    listener: (event: WorkerMessageEvent) => void,
  ) => void;
};

type PendingRequest = {
  readonly resolve: (value: unknown) => void;
  readonly reject: (error: ErrorEnvelopeDto) => void;
};

export class WasmClient {
  private nextCorrelationId = 1;
  private readonly pending = new Map<number, PendingRequest>();
  private readonly onMessage = (event: WorkerMessageEvent): void => {
    this.receive(event.data);
  };

  public constructor(private readonly transport: WasmTransport) {
    this.transport.addEventListener("message", this.onMessage);
  }

  public dispose(): void {
    this.transport.removeEventListener("message", this.onMessage);
    for (const request of this.pending.values()) {
      request.reject(protocolError("Wasm client disposed before response"));
    }
    this.pending.clear();
  }

  public loadGroup(
    groupId: string,
    viewer: ViewerContextDto,
    templateJson: string,
    opsJsonl: string,
  ): Promise<JsonObject> {
    return this.request({ kind: "loadGroup", groupId, viewer, templateJson, opsJsonl });
  }

  public projection(groupId: string, viewer: ViewerContextDto): Promise<ProjectionDto> {
    return this.request({ kind: "projection", groupId, viewer }) as Promise<ProjectionDto>;
  }

  public entityDetail(
    groupId: string,
    viewer: ViewerContextDto,
    entityId: string,
  ): Promise<EntityDetailDto> {
    return this.request({ kind: "entityDetail", groupId, viewer, entityId }) as Promise<EntityDetailDto>;
  }

  public submitOps(
    groupId: string,
    viewer: ViewerContextDto,
    opsJson: string,
  ): Promise<JsonObject> {
    return this.request({ kind: "submitOps", groupId, viewer, opsJson });
  }

  public search(
    groupId: string,
    viewer: ViewerContextDto,
    query: JsonObject,
  ): Promise<readonly SearchHitDto[]> {
    return this.request({ kind: "search", groupId, viewer, query });
  }

  public queryPaths(
    groupId: string,
    viewer: ViewerContextDto,
    request: JsonObject,
  ): Promise<JsonObject> {
    return this.request({ kind: "queryPaths", groupId, viewer, request });
  }

  public queryNeighborhood(
    groupId: string,
    viewer: ViewerContextDto,
    request: JsonObject,
  ): Promise<JsonObject> {
    return this.request({ kind: "queryNeighborhood", groupId, viewer, request });
  }

  public intakeStageRecord(payloadJson: string, stagedAtMs: number): Promise<JsonObject> {
    return this.request({ kind: "intakeStageRecord", payloadJson, stagedAtMs });
  }

  public intakeBuildDecision(requestJson: string): Promise<JsonObject> {
    return this.request({ kind: "intakeBuildDecision", requestJson });
  }

  public intakeValidateRecord(groupId: string, recordJson: string): Promise<JsonObject> {
    return this.request({ kind: "intakeValidateRecord", groupId, recordJson });
  }

  public intakeDedupCheck(recordJson: string, existingKeysJson: string): Promise<JsonObject> {
    return this.request({ kind: "intakeDedupCheck", recordJson, existingKeysJson });
  }

  public intakeNearDuplicates(
    groupId: string,
    viewer: ViewerContextDto,
    requestJson: string,
  ): Promise<readonly JsonObject[]> {
    return this.request({ kind: "intakeNearDuplicates", groupId, viewer, requestJson });
  }

  public toErrorEnvelope(error: unknown): ErrorEnvelopeDto {
    if (this.isErrorEnvelope(error)) {
      return error;
    }
    return protocolError(error instanceof Error ? error.message : "Unknown wasm client error");
  }

  private request<T = JsonObject>(request: WorkerRequestInput): Promise<T> {
    const correlationId = this.nextCorrelationId;
    this.nextCorrelationId += 1;
    const workerRequest = { ...request, correlationId } as WorkerRequest;
    return new Promise<T>((resolve, reject) => {
      this.pending.set(correlationId, {
        resolve: (value) => resolve(value as T),
        reject,
      });
      this.transport.postMessage(workerRequest);
    });
  }

  private receive(response: WorkerResponse): void {
    const request = this.pending.get(response.correlationId);
    if (request === undefined) {
      return;
    }
    this.pending.delete(response.correlationId);
    if ("err" in response) {
      request.reject(response.err);
      return;
    }
    request.resolve(response.ok);
  }

  private isErrorEnvelope(error: unknown): error is ErrorEnvelopeDto {
    if (typeof error !== "object" || error === null) {
      return false;
    }
    const maybe = error as Partial<ErrorEnvelopeDto>;
    return typeof maybe.code === "string" && typeof maybe.message === "string";
  }
}
