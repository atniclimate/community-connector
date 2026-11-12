import { describe, expect, it } from "vitest";
import type { Action } from "./actions";
import { parseMirror, parsePath } from "./analysis";
import { loadIntroductions, loadNeighborhood, runRouting } from "./effects";
import { reduce } from "./reducer";
import { selectMirrorView, selectRoutingResults, selectSelfEntityId } from "./selectors";
import {
  createInitialState,
  type AnalysisName,
  type AppState,
  type ProjectionDto,
  type RecordMirrorDto,
  type RequestIdentity,
  type RouteCandidateDto,
} from "./state";
import { createStore } from "./store";
import type { WasmClient } from "../wasm/client";

const PROJECTION: ProjectionDto = {
  revision: 1,
  entities: [
    { id: "e-other", kind: "person", owner_is_viewer: false },
    { id: "e-self", kind: "person", owner_is_viewer: true },
    { id: "e-rest", kind: "person", owner_is_viewer: false, attributes: { capacity: { type: "enum", value: "resting" } } },
    { id: "e-open", kind: "person", owner_is_viewer: false, attributes: { capacity: { type: "enum", value: "open" } } },
  ],
  edges: [],
};

const NEXT_PROJECTION: ProjectionDto = { revision: 2, entities: [], edges: [] };

function loaded(projection: ProjectionDto = PROJECTION): AppState {
  const scoped = reduce(createInitialState(), {
    kind: "groupLoadRequested",
    groupId: "group-alpha",
    viewer: { kind: "person", person: "p-self" },
  });
  return reduce(scoped, { kind: "projectionReceived", projection, revision: projection.revision ?? 1 });
}

function req(state: AppState, requestId: number): RequestIdentity {
  return {
    requestId,
    sessionId: state.session.sessionId,
    groupId: state.session.groupId ?? "",
    viewer: state.session.viewer,
  };
}

function candidate(entity: string): RouteCandidateDto {
  return { entity, hits: [{ entity, kind: "person", matched_attr: "offering", snippet: "water" }], path: null };
}

const MIRROR: RecordMirrorDto = {
  owner: "p-self",
  entities: {
    "e-self": {
      kind: "person",
      presence: {
        visibility: "network",
        effective_tier: "T2",
        ceiling: "group",
        audiences: { public: false, network: false, group: true, trusted: true },
        tier_capped: true,
        ceiling_blocks: ["network"],
      },
      attributes: {
        offering: {
          visibility: "public",
          effective_tier: "T1",
          ceiling: "network",
          audiences: {},
          tier_capped: true,
          ceiling_blocks: ["public"],
        },
        seeking: {
          visibility: "group",
          effective_tier: "T2",
          ceiling: "group",
          audiences: {},
          tier_capped: false,
          ceiling_blocks: [],
        },
      },
    },
  },
};

describe("routing reducer", () => {
  it("tracks the route lifecycle and keeps prior results while pending", () => {
    const base = loaded();
    const pending = reduce(base, { kind: "routeRequested", query: "water", request: req(base, 1) });
    expect(pending.routing).toMatchObject({ query: "water", status: "pending", request: { requestId: 1 } });

    const ready = reduce(pending, {
      kind: "routeSucceeded",
      query: "water",
      candidates: [candidate("e-other")],
      request: req(base, 1),
    });
    expect(ready.routing).toMatchObject({ status: "ready", request: null, error: null });
    expect(ready.routing.result).toHaveLength(1);

    const again = reduce(ready, { kind: "routeRequested", query: "water", request: req(base, 2) });
    expect(again.routing.result).toHaveLength(1);

    const failed = reduce(again, {
      kind: "routeFailed",
      query: "water",
      error: { code: "ProtocolError", message: "bad" },
      request: req(base, 2),
    });
    expect(failed.routing).toMatchObject({ status: "error", result: null, request: null });
    expect(failed.routing.error?.code).toBe("ProtocolError");

    expect(reduce(failed, { kind: "routeCleared" }).routing).toEqual(createInitialState().routing);
  });

  it("drops stale route responses by query, request, and session", () => {
    const base = loaded();
    const pending = reduce(base, { kind: "routeRequested", query: "water", request: req(base, 1) });
    const ok = (query: string, request: RequestIdentity): Action => ({
      kind: "routeSucceeded",
      query,
      candidates: [],
      request,
    });
    const fail = (request: RequestIdentity): Action => ({
      kind: "routeFailed",
      query: "water",
      error: { code: "x", message: "x" },
      request,
    });

    expect(reduce(pending, ok("fire", req(base, 1)))).toBe(pending);
    expect(reduce(pending, ok("water", req(base, 9)))).toBe(pending);
    expect(reduce(pending, fail(req(base, 9)))).toBe(pending);
    expect(reduce(pending, ok("water", { ...req(base, 1), sessionId: 99 }))).toBe(pending);
    expect(reduce(pending, ok("water", { ...req(base, 1), viewer: { kind: "anonymous" } }))).toBe(pending);
  });

  it("resets on a new projection keeping the query, and fully on a group load", () => {
    const base = loaded();
    const ready = reduce(reduce(base, { kind: "routeRequested", query: "water", request: req(base, 1) }), {
      kind: "routeSucceeded",
      query: "water",
      candidates: [candidate("e-other")],
      request: req(base, 1),
    });
    const next = reduce(ready, { kind: "projectionReceived", projection: NEXT_PROJECTION, revision: 2 });
    expect(next.routing).toEqual({ ...createInitialState().routing, query: "water" });

    const reloaded = reduce(ready, { kind: "groupLoadRequested", groupId: "group-beta", viewer: { kind: "anonymous" } });
    expect(reloaded.routing).toEqual(createInitialState().routing);
  });
});

describe("analysis reducer", () => {
  const results: { readonly [K in AnalysisName]: Extract<Action, { kind: "analysisSucceeded"; analysis: K }>["result"] } = {
    neighborhood: { center: "e-self", layers: [["e-self"], ["e-other"]] },
    continuance: { articulation_points: {}, bridges: {}, components: {} },
    introductions: [],
    mirror: MIRROR,
  };

  for (const analysis of Object.keys(results) as AnalysisName[]) {
    it(`runs the ${analysis} lifecycle and drops stale responses`, () => {
      const base = loaded();
      const pending = reduce(base, { kind: "analysisRequested", analysis, request: req(base, 5) });
      expect(pending.analysis[analysis]).toMatchObject({ status: "pending", request: { requestId: 5 } });

      const success = (request: RequestIdentity): Action =>
        ({ kind: "analysisSucceeded", analysis, result: results[analysis], request }) as Action;
      const failure = (request: RequestIdentity): Action => ({
        kind: "analysisFailed",
        analysis,
        error: { code: "not_found", message: "not found" },
        request,
      });

      expect(reduce(pending, success(req(base, 4)))).toBe(pending);
      expect(reduce(pending, failure(req(base, 4)))).toBe(pending);
      expect(reduce(pending, success({ ...req(base, 5), sessionId: 99 }))).toBe(pending);
      expect(reduce(pending, success({ ...req(base, 5), groupId: "group-beta" }))).toBe(pending);

      const ready = reduce(pending, success(req(base, 5)));
      expect(ready.analysis[analysis]).toEqual({ status: "ready", result: results[analysis], error: null, request: null });

      const failed = reduce(pending, failure(req(base, 5)));
      expect(failed.analysis[analysis]).toMatchObject({ status: "error", result: null, request: null });
      expect(failed.analysis[analysis].error?.code).toBe("not_found");

      const others = (Object.keys(results) as AnalysisName[]).filter((name) => name !== analysis);
      for (const other of others) {
        expect(ready.analysis[other]).toBe(base.analysis[other]);
      }
      expect(reduce(ready, { kind: "projectionReceived", projection: NEXT_PROJECTION, revision: 2 }).analysis)
        .toEqual(createInitialState().analysis);
      expect(reduce(ready, { kind: "groupLoadRequested", groupId: "group-alpha", viewer: { kind: "anonymous" } }).analysis)
        .toEqual(createInitialState().analysis);
    });
  }
});

describe("relatives selectors", () => {
  it("finds the viewer's own entity or null", () => {
    expect(selectSelfEntityId(loaded())).toBe("e-self");
    expect(selectSelfEntityId(loaded({ revision: 1, entities: [{ id: "e-other" }], edges: [] }))).toBeNull();
    expect(selectSelfEntityId(createInitialState())).toBeNull();
  });

  it("orders visibly resting candidates last and keeps core order otherwise", () => {
    const base = loaded();
    const ready = reduce(reduce(base, { kind: "routeRequested", query: "water", request: req(base, 1) }), {
      kind: "routeSucceeded",
      query: "water",
      candidates: [candidate("e-rest"), candidate("e-open"), candidate("e-other"), candidate("e-gone")],
      request: req(base, 1),
    });
    const ordered = selectRoutingResults(ready);
    expect(ordered.map((c) => c.entity)).toEqual(["e-open", "e-other", "e-gone", "e-rest"]);
    expect(ordered.map((c) => c.resting)).toEqual([false, false, false, true]);
    expect(selectRoutingResults(base)).toEqual([]);
  });

  it("writes a plain-language line for every tier-capped value", () => {
    const base = loaded();
    const ready = reduce(base, {
      kind: "analysisSucceeded",
      analysis: "mirror",
      result: MIRROR,
      request: req(base, 1),
    });
    expect(selectMirrorView(ready).gaps).toEqual([]);

    const pending = reduce(base, { kind: "analysisRequested", analysis: "mirror", request: req(base, 1) });
    const view = selectMirrorView(reduce(pending, { kind: "analysisSucceeded", analysis: "mirror", result: MIRROR, request: req(base, 1) }));
    expect(view.status).toBe("ready");
    expect(view.gaps).toEqual([
      { entityId: "e-self", attribute: null, text: "You chose network; this record's tier holds it at group." },
      {
        entityId: "e-self",
        attribute: "offering",
        text: "For offering, you chose public; this record's tier holds it at network.",
      },
    ]);
  });
});

describe("relatives parsers", () => {
  it("accepts null or a well-formed path and rejects malformed ones", () => {
    expect(parsePath(null)).toEqual({ value: null });
    expect(parsePath({ nodes: ["a", "b"], edges: ["x"], cost: 1 })).toEqual({ value: { nodes: ["a", "b"], edges: ["x"], cost: 1 } });
    expect(parsePath({ nodes: "a" })).toHaveProperty("error.code", "ProtocolError");
    expect(parseMirror({ owner: null, entities: {} })).toEqual({ value: { owner: null, entities: {} } });
    expect(parseMirror({ entities: { e: { presence: {} } } })).toHaveProperty("error.code", "ProtocolError");
  });
});

type Call = { readonly method: string; readonly args: readonly unknown[] };

function fakeClient(responses: Record<string, (...args: unknown[]) => unknown>): { client: WasmClient; calls: Call[] } {
  const calls: Call[] = [];
  const client = new Proxy({} as WasmClient, {
    get: (_target, method: string) => {
      if (method === "toErrorEnvelope") {
        return (error: unknown) => error;
      }
      return async (...args: unknown[]) => {
        calls.push({ method, args });
        const respond = responses[method];
        if (respond === undefined) {
          throw new Error(`unexpected call ${method}`);
        }
        return respond(...args);
      };
    },
  });
  return { client, calls };
}

describe("relatives effects", () => {
  it("routes from the viewer's own entity to each search candidate", async () => {
    const store = createStore(loaded(), { devFreeze: false });
    const { client, calls } = fakeClient({
      search: () => [
        { entity: "e-self", kind: "person", matched_attr: "offering", snippet: "water" },
        { entity: "e-open", kind: "person", matched_attr: "offering", snippet: "water" },
        { entity: "e-open", kind: "person", matched_attr: "specialties", snippet: "water" },
      ],
      queryPaths: () => ({ nodes: ["e-self", "e-open"], edges: ["x"], cost: 1 }),
    });
    await runRouting(store, client, "  water ");

    expect(calls.map((call) => call.method)).toEqual(["search", "queryPaths"]);
    expect(calls[1]?.args[2]).toEqual({
      from: "e-self",
      to: "e-open",
      constraints: { allowed_edge_kinds: null, max_hops: 4, weighted: false, respect_direction: false },
    });
    const routing = store.getState().routing;
    expect(routing.status).toBe("ready");
    expect(routing.result).toEqual([
      {
        entity: "e-open",
        hits: [
          { entity: "e-open", kind: "person", matched_attr: "offering", snippet: "water" },
          { entity: "e-open", kind: "person", matched_attr: "specialties", snippet: "water" },
        ],
        path: { nodes: ["e-self", "e-open"], edges: ["x"], cost: 1 },
      },
    ]);
  });

  it("surfaces a malformed path as a route failure", async () => {
    const store = createStore(loaded(), { devFreeze: false });
    const { client } = fakeClient({
      search: () => [{ entity: "e-open", kind: "person", matched_attr: "offering", snippet: "water" }],
      queryPaths: () => ({ nodes: 3 }),
    });
    await runRouting(store, client, "water");
    expect(store.getState().routing).toMatchObject({ status: "error", error: { code: "ProtocolError" } });
  });

  it("centers neighborhood and introductions on the viewer's own entity", async () => {
    const store = createStore(loaded(), { devFreeze: false });
    const { client, calls } = fakeClient({
      queryNeighborhood: () => ({ center: "e-self", layers: [["e-self"]] }),
      suggestIntroductions: () => [],
    });
    await loadNeighborhood(store, client);
    await loadIntroductions(store, client);
    expect(calls.map((call) => call.args[2])).toEqual([{ center: "e-self", hops: 3 }, { center: "e-self" }]);
    expect(store.getState().analysis.neighborhood.status).toBe("ready");
    expect(store.getState().analysis.introductions.status).toBe("ready");
  });

  it("requests nothing centered when the viewer has no own entity", async () => {
    const store = createStore(loaded({ revision: 1, entities: [{ id: "e-other" }], edges: [] }), { devFreeze: false });
    const { client, calls } = fakeClient({});
    await loadNeighborhood(store, client);
    await loadIntroductions(store, client);
    expect(calls).toEqual([]);
    expect(store.getState().analysis).toEqual(createInitialState().analysis);
  });
});
