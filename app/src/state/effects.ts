import type { Store } from "./store";
import type {
  AppState,
  JsonObject,
  KindMeta,
  RequestIdentity,
  ShapeName,
  ViewerContextDto,
} from "./state";
import type { WasmClient } from "../wasm/client";
import { deriveTheme } from "../theme/derive";
import type { GroupTemplateDto, GroupTemplateKindDto } from "../theme/tokens";
import { parseSearchHits } from "./search";

/** Attribute-hit cap sent with every search query (existing cn-api contract). */
export const SEARCH_RESULT_LIMIT = 25;
let nextRequestId = 1;

const SHAPES: readonly ShapeName[] = ["sphere", "cube", "octahedron", "tetrahedron", "torus", "cone"];

function revisionFromProjection(projection: JsonObject): number {
  const revision = projection.revision;
  return typeof revision === "number" ? revision : 0;
}

function isShapeName(value: string | undefined): value is ShapeName {
  return SHAPES.includes(value as ShapeName);
}

function kindMetaFromTemplateKind(kind: GroupTemplateKindDto): KindMeta {
  return {
    shape: isShapeName(kind.shape) ? kind.shape : "sphere",
    label: kind.label ?? kind.id,
    colorRole: kind.color_role,
  };
}

export function kindMetaFromTemplate(template: GroupTemplateDto): Readonly<Record<string, KindMeta>> {
  return Object.fromEntries(
    template.kinds.map((kind) => [kind.id, kindMetaFromTemplateKind(kind)]),
  );
}

export function requestIdentity(state: AppState): RequestIdentity | null {
  if (state.session.groupId === null) {
    return null;
  }
  const identity = {
    requestId: nextRequestId,
    sessionId: state.session.sessionId,
    groupId: state.session.groupId,
    viewer: state.session.viewer,
  };
  nextRequestId += 1;
  return identity;
}

/** Implements docs/blueprints/app-state.md effects and ADR-003 D5 worker dispatch. */
export async function loadGroup(
  store: Store,
  client: WasmClient,
  groupId: string,
  viewer: ViewerContextDto,
  templateJson: string,
  opsJsonl = "",
): Promise<void> {
  store.dispatch({ kind: "groupLoadRequested", groupId, viewer });
  try {
    await client.loadGroup(groupId, viewer, templateJson, opsJsonl);
    const template = JSON.parse(templateJson) as GroupTemplateDto;
    const derived = deriveTheme(template);
    store.dispatch({
      kind: "themeDerived",
      theme: derived.theme,
      report: derived.report,
      kindMeta: kindMetaFromTemplate(template),
    });
    store.dispatch({ kind: "groupLoadSucceeded", groupId });
    await refreshProjection(store, client);
  } catch (error) {
    store.dispatch({ kind: "groupLoadFailed", groupId, error: client.toErrorEnvelope(error) });
  }
}

/** Implements docs/blueprints/app-state.md promise-to-dispatch boundary. */
export async function refreshProjection(store: Store, client: WasmClient): Promise<void> {
  const state = store.getState();
  const request = requestIdentity(state);
  if (request === null) {
    return;
  }
  try {
    const projection = await client.projection(request.groupId, request.viewer);
    store.dispatch({
      kind: "projectionReceived",
      projection,
      revision: revisionFromProjection(projection),
    });
  } catch (error) {
    store.dispatch({ kind: "errorSurfaced", error: client.toErrorEnvelope(error) });
  }
}

/**
 * Runs the existing cn-api `search` operation (attribute hits only, no new
 * query contract - P1.4) and folds the outcome into the state machine (I4).
 * Stale responses are dropped by the reducer's query-identity guard.
 */
export async function runSearch(store: Store, client: WasmClient, query: string): Promise<void> {
  const state = store.getState();
  const request = requestIdentity(state);
  if (request === null) {
    return;
  }
  const trimmed = query.trim();
  if (trimmed === "") {
    store.dispatch({ kind: "searchCleared" });
    return;
  }
  store.dispatch({ kind: "searchRequested", query: trimmed, request });
  try {
    const response = await client.search(request.groupId, request.viewer, {
      text: trimmed,
      limit: SEARCH_RESULT_LIMIT,
    });
    const parsed = parseSearchHits(response);
    if ("error" in parsed) {
      store.dispatch({ kind: "searchFailed", query: trimmed, request, error: parsed.error });
      return;
    }
    store.dispatch({ kind: "searchSucceeded", query: trimmed, request, hits: parsed.hits });
  } catch (error) {
    store.dispatch({
      kind: "searchFailed",
      query: trimmed,
      request,
      error: client.toErrorEnvelope(error),
    });
  }
}

/** Implements docs/blueprints/app-state.md stale detail tagging requirement. */
export async function openDetail(
  store: Store,
  client: WasmClient,
  entityId: string,
): Promise<void> {
  const state = store.getState();
  const request = requestIdentity(state);
  if (request === null) {
    return;
  }
  store.dispatch({ kind: "entityFocused", entityId });
  store.dispatch({ kind: "detailRequested", entityId, request });
  try {
    const detail = await client.entityDetail(request.groupId, request.viewer, entityId);
    store.dispatch({ kind: "detailReceived", detail, entityId, request });
  } catch (error) {
    store.dispatch({
      kind: "detailFailed",
      entityId,
      request,
      error: client.toErrorEnvelope(error),
    });
  }
}
