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
import type { Action } from "./actions";
import type { AnalysisName, PathDto } from "./state";
import {
  candidatesFromHits,
  parseContinuance,
  parseIntroductions,
  parseMirror,
  parseNeighborhood,
  parsePath,
  type Parsed,
} from "./analysis";
import { selectSelfEntityId } from "./selectors";

/** Longest "who can help" chain from the viewer's own entity. */
export const ROUTE_MAX_HOPS = 4;
/** Rings of the viewer's neighborhood for the ring layout. */
export const NEIGHBORHOOD_HOPS = 3;
const ROUTE_CONSTRAINTS = {
  allowed_edge_kinds: null,
  max_hops: ROUTE_MAX_HOPS,
  weighted: false,
  respect_direction: false,
};

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

/**
 * "Who can help": runs the existing core search, then a shortest path from
 * the viewer's own entity to each candidate (unweighted, direction ignored,
 * at most ROUTE_MAX_HOPS). With no own entity in the projection every
 * candidate carries a null path. Stale responses are dropped by the reducer.
 */
export async function runRouting(store: Store, client: WasmClient, query: string): Promise<void> {
  const state = store.getState();
  const request = requestIdentity(state);
  if (request === null) {
    return;
  }
  const trimmed = query.trim();
  if (trimmed === "") {
    store.dispatch({ kind: "routeCleared" });
    return;
  }
  const self = selectSelfEntityId(state);
  store.dispatch({ kind: "routeRequested", query: trimmed, request });
  const pathTo = async (to: string): Promise<PathDto | null> => {
    if (self === null) {
      return null;
    }
    const raw = await client.queryPaths(request.groupId, request.viewer, {
      from: self,
      to,
      constraints: ROUTE_CONSTRAINTS,
    });
    const parsed = parsePath(raw);
    if ("error" in parsed) {
      throw parsed.error;
    }
    return parsed.value;
  };
  try {
    const hits = parseSearchHits(
      await client.search(request.groupId, request.viewer, { text: trimmed, limit: SEARCH_RESULT_LIMIT }),
    );
    if ("error" in hits) {
      store.dispatch({ kind: "routeFailed", query: trimmed, request, error: hits.error });
      return;
    }
    const candidates = await Promise.all(
      candidatesFromHits(hits.hits, self).map(async (found) => ({ ...found, path: await pathTo(found.entity) })),
    );
    store.dispatch({ kind: "routeSucceeded", query: trimmed, request, candidates });
  } catch (error) {
    store.dispatch({ kind: "routeFailed", query: trimmed, request, error: client.toErrorEnvelope(error) });
  }
}

async function runAnalysis<T>(
  store: Store,
  client: WasmClient,
  analysis: AnalysisName,
  call: (request: RequestIdentity) => Promise<unknown>,
  parse: (value: unknown) => Parsed<T>,
  succeeded: (result: T, request: RequestIdentity) => Action,
): Promise<void> {
  const request = requestIdentity(store.getState());
  if (request === null) {
    return;
  }
  store.dispatch({ kind: "analysisRequested", analysis, request });
  try {
    const parsed = parse(await call(request));
    store.dispatch(
      "error" in parsed
        ? { kind: "analysisFailed", analysis, request, error: parsed.error }
        : succeeded(parsed.value, request),
    );
  } catch (error) {
    store.dispatch({ kind: "analysisFailed", analysis, request, error: client.toErrorEnvelope(error) });
  }
}

/** The viewer's neighborhood rings for the ring layout. No own entity in
 * the projection means nothing to center on, so nothing is requested. */
export async function loadNeighborhood(store: Store, client: WasmClient): Promise<void> {
  const center = selectSelfEntityId(store.getState());
  if (center === null) {
    return;
  }
  await runAnalysis(
    store,
    client,
    "neighborhood",
    (r) => client.queryNeighborhood(r.groupId, r.viewer, { center, hops: NEIGHBORHOOD_HOPS }),
    parseNeighborhood,
    (result, request) => ({ kind: "analysisSucceeded", analysis: "neighborhood", result, request }),
  );
}

/** Articulation points and bridge ties, scoped by the core to the viewer. */
export async function loadContinuance(store: Store, client: WasmClient): Promise<void> {
  await runAnalysis(
    store,
    client,
    "continuance",
    (r) => client.graphContinuance(r.groupId, r.viewer, {}),
    parseContinuance,
    (result, request) => ({ kind: "analysisSucceeded", analysis: "continuance", result, request }),
  );
}

/** Introduction suggestions centered on the viewer's own entity; nothing is
 * requested without one (the core would answer not_found anyway). */
export async function loadIntroductions(store: Store, client: WasmClient): Promise<void> {
  const center = selectSelfEntityId(store.getState());
  if (center === null) {
    return;
  }
  await runAnalysis(
    store,
    client,
    "introductions",
    (r) => client.suggestIntroductions(r.groupId, r.viewer, { center }),
    parseIntroductions,
    (result, request) => ({ kind: "analysisSucceeded", analysis: "introductions", result, request }),
  );
}

/** What each audience receives of the viewer's own records. */
export async function loadMirror(store: Store, client: WasmClient): Promise<void> {
  await runAnalysis(
    store,
    client,
    "mirror",
    (r) => client.recordMirror(r.groupId, r.viewer),
    parseMirror,
    (result, request) => ({ kind: "analysisSucceeded", analysis: "mirror", result, request }),
  );
}
