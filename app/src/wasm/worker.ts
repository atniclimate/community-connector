import init, { CnApi } from "../../../core/crates/cn-wasm/pkg/cn_wasm.js";
import type { ErrorEnvelopeDto, JsonObject } from "../state/state";
import type { Envelope, WorkerRequest, WorkerResponse } from "./protocol";
import { protocolError } from "./protocol";

let apiPromise: Promise<CnApi> | null = null;

function viewerJson(request: { readonly viewer: JsonObject }): string {
  return JSON.stringify(request.viewer);
}

function parseEnvelope<T>(json: string): Envelope<T> {
  try {
    const parsed = JSON.parse(json) as Envelope<T>;
    if ("ok" in parsed || "err" in parsed) {
      return parsed;
    }
    return { err: protocolError("Wasm envelope missing ok or err") };
  } catch (error) {
    return { err: protocolError(error instanceof Error ? error.message : "Invalid wasm JSON") };
  }
}

async function api(): Promise<CnApi> {
  if (apiPromise === null) {
    apiPromise = init().then(() => new CnApi());
  }
  return apiPromise;
}

function runRequest(instance: CnApi, request: WorkerRequest): string {
  switch (request.kind) {
    case "loadGroup":
      return loadGroup(instance, request);
    case "projection":
      return instance.projection(request.groupId, viewerJson(request));
    case "entityDetail":
      return instance.entity_detail(request.groupId, viewerJson(request), request.entityId);
    case "submitOps":
      return instance.submit_ops(request.groupId, viewerJson(request), request.opsJson, Date.now());
    case "search":
      return instance.search(request.groupId, viewerJson(request), JSON.stringify(request.query));
    case "queryPaths":
      return instance.query_paths(request.groupId, viewerJson(request), JSON.stringify(request.request));
    case "queryNeighborhood":
      return instance.query_neighborhood(request.groupId, viewerJson(request), JSON.stringify(request.request));
    default:
      return JSON.stringify({ err: protocolError("Unknown worker request") });
  }
}

function loadGroup(instance: CnApi, request: Extract<WorkerRequest, { readonly kind: "loadGroup" }>): string {
  const begin = parseEnvelope<JsonObject>(
    instance.load_group_begin(request.groupId, viewerJson(request), request.templateJson),
  );
  if ("err" in begin) {
    return JSON.stringify(begin);
  }
  if (request.opsJsonl.length > 0) {
    const chunk = parseEnvelope<JsonObject>(instance.load_ops_chunk(request.groupId, request.opsJsonl));
    if ("err" in chunk) {
      return JSON.stringify(chunk);
    }
  }
  return instance.load_group_commit(request.groupId, Date.now());
}

type WorkerSuccess = Extract<WorkerResponse, { readonly ok: unknown }>["ok"];

function toResponse(correlationId: number, envelope: Envelope<WorkerSuccess>): WorkerResponse {
  if ("err" in envelope) {
    return { correlationId, err: envelope.err };
  }
  return { correlationId, ok: envelope.ok };
}

function thrownError(error: unknown): ErrorEnvelopeDto {
  return protocolError(error instanceof Error ? error.message : "Unknown worker failure");
}

async function handleMessage(event: MessageEvent<WorkerRequest>): Promise<void> {
  const correlationId = event.data.correlationId;
  try {
    const instance = await api();
    const envelope = parseEnvelope<WorkerSuccess>(runRequest(instance, event.data));
    postMessage(toResponse(correlationId, envelope));
  } catch (error) {
    postMessage({ correlationId, err: thrownError(error) } satisfies WorkerResponse);
  }
}

addEventListener("message", (event: MessageEvent<WorkerRequest>) => {
  void handleMessage(event);
});
