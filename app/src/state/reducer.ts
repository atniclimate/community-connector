import type { Action } from "./actions";
import type { AppState } from "./state";
import { initialSearchState } from "./state";
import type { RequestIdentity } from "./state";

function assertNever(value: never): never {
  throw new Error(`Unhandled action: ${JSON.stringify(value)}`);
}

function resetView(): AppState["view"] {
  return {
    mode: "overview",
    focusedEntityId: null,
    hoveredEntityId: null,
    storyId: null,
    storyStep: 0,
  };
}

function focusView(entityId: string): AppState["view"] {
  return {
    mode: "focus",
    focusedEntityId: entityId,
    hoveredEntityId: null,
    storyId: null,
    storyStep: 0,
  };
}

function storyView(storyId: string, step: number): AppState["view"] {
  return {
    mode: "story",
    focusedEntityId: null,
    hoveredEntityId: null,
    storyId,
    storyStep: Math.max(0, step),
  };
}

function requestMatchesSession(state: AppState, request: RequestIdentity): boolean {
  return (
    request.sessionId === state.session.sessionId &&
    request.groupId === state.session.groupId &&
    request.viewer === state.session.viewer
  );
}

function sameRequest(left: RequestIdentity | null, right: RequestIdentity): boolean {
  return left !== null && left.requestId === right.requestId;
}

/** Implements docs/blueprints/app-state.md state machine and ADR-003 D3 staleness. */
export function reduce(state: AppState, action: Action): AppState {
  switch (action.kind) {
    case "groupLoadRequested":
      return {
        ...state,
        session: {
          groupId: action.groupId,
          viewer: action.viewer,
          sessionId: state.session.sessionId + 1,
          revision: 0,
          loadState: "loading",
          lastError: null,
        },
        view: resetView(),
        ui: { ...state.ui, detailEntityId: null },
        data: {
          projection: null,
          detail: null,
          detailRequest: null,
          detailError: null,
          kindMeta: {},
        },
        search: initialSearchState(),
        theme: { resolved: null, report: null },
      };
    case "groupLoadSucceeded":
      return {
        ...state,
        session: { ...state.session, groupId: action.groupId, loadState: "ready" },
      };
    case "groupLoadFailed":
      return {
        ...state,
        session: {
          ...state.session,
          groupId: action.groupId,
          loadState: "error",
          lastError: action.error,
        },
      };
    case "projectionReceived":
      if (action.revision <= state.session.revision && state.data.projection !== null) {
        return state;
      }
      return {
        ...state,
        session: { ...state.session, revision: action.revision, loadState: "ready" },
        data: {
          ...state.data,
          projection: action.projection,
          detail: null,
          detailRequest: null,
          detailError: null,
        },
        // Hits computed against the previous projection are stale viewer-scoped
        // data; the query survives so the viewer can re-run it.
        search: { ...initialSearchState(), query: state.search.query },
      };
    case "themeDerived":
      return {
        ...state,
        // Theme derivation is revision-independent in v0: template changes arrive as a new group load.
        theme: { resolved: action.theme, report: action.report },
        data: { ...state.data, kindMeta: action.kindMeta },
      };
    case "entityHovered":
      return {
        ...state,
        view: { ...state.view, hoveredEntityId: action.entityId },
      };
    case "entityFocused":
      return {
        ...state,
        view: focusView(action.entityId),
        ui: { ...state.ui, detailEntityId: action.entityId },
        data: { ...state.data, detail: null, detailRequest: null, detailError: null },
      };
    case "entityFocusCleared":
      return { ...state, view: resetView() };
    case "detailRequested":
      if (!requestMatchesSession(state, action.request) || state.ui.detailEntityId !== action.entityId) {
        return state;
      }
      return {
        ...state,
        data: {
          ...state.data,
          detail: null,
          detailRequest: { ...action.request, entityId: action.entityId },
          detailError: null,
        },
      };
    case "detailReceived":
      if (
        !requestMatchesSession(state, action.request) ||
        state.ui.detailEntityId !== action.entityId ||
        state.data.detailRequest?.entityId !== action.entityId ||
        !sameRequest(state.data.detailRequest, action.request)
      ) {
        return state;
      }
      return {
        ...state,
        data: { ...state.data, detail: action.detail, detailRequest: null, detailError: null },
      };
    case "detailFailed":
      if (
        !requestMatchesSession(state, action.request) ||
        state.ui.detailEntityId !== action.entityId ||
        state.data.detailRequest?.entityId !== action.entityId ||
        !sameRequest(state.data.detailRequest, action.request)
      ) {
        return state;
      }
      return {
        ...state,
        session: { ...state.session, lastError: action.error },
        data: { ...state.data, detailRequest: null, detailError: action.error },
      };
    case "detailCleared":
      return {
        ...state,
        ui: { ...state.ui, detailEntityId: null },
        data: { ...state.data, detail: null, detailRequest: null, detailError: null },
      };
    case "storyEntered":
      return { ...state, view: storyView(action.storyId, action.step) };
    case "storyStepped":
      if (state.view.mode !== "story" || state.view.storyId === null) {
        return state;
      }
      return {
        ...state,
        view: storyView(state.view.storyId, action.step),
      };
    case "storyExited":
      return { ...state, view: resetView() };
    case "legendToggled":
      return { ...state, ui: { ...state.ui, legendOpen: !state.ui.legendOpen } };
    case "sidebarToggled":
      return { ...state, ui: { ...state.ui, sidebarOpen: !state.ui.sidebarOpen } };
    case "reducedMotionChanged":
      return {
        ...state,
        ui: { ...state.ui, reducedMotion: action.reducedMotion },
      };
    case "qualityTierChanged":
      return {
        ...state,
        ui: { ...state.ui, qualityTier: action.tier },
      };
    case "searchRequested":
      return {
        ...state,
        search: {
          query: action.query,
          status: "pending",
          hits: state.search.hits,
          error: null,
          request: action.request,
        },
      };
    case "searchSucceeded":
      // A response for a query the viewer has already replaced is stale; drop it.
      if (
        action.query !== state.search.query ||
        !requestMatchesSession(state, action.request) ||
        !sameRequest(state.search.request, action.request)
      ) {
        return state;
      }
      return {
        ...state,
        search: {
          ...state.search,
          status: "ready",
          hits: action.hits,
          error: null,
          request: null,
        },
      };
    case "searchFailed":
      if (
        action.query !== state.search.query ||
        !requestMatchesSession(state, action.request) ||
        !sameRequest(state.search.request, action.request)
      ) {
        return state;
      }
      return {
        ...state,
        search: { ...state.search, status: "error", hits: [], error: action.error, request: null },
      };
    case "searchCleared":
      return { ...state, search: initialSearchState() };
    case "errorSurfaced":
      return {
        ...state,
        session: { ...state.session, lastError: action.error },
      };
    case "errorDismissed":
      return {
        ...state,
        session: { ...state.session, lastError: null },
      };
    default:
      return assertNever(action);
  }
}
