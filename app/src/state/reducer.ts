import type { Action } from "./actions";
import type { AppState } from "./state";

function assertNever(value: never): never {
  throw new Error(`Unhandled action: ${JSON.stringify(value)}`);
}

function resetView(): AppState["view"] {
  return {
    mode: "overview",
    focusedEntityId: null,
    storyId: null,
    storyStep: 0,
  };
}

function focusView(entityId: string): AppState["view"] {
  return {
    mode: "focus",
    focusedEntityId: entityId,
    storyId: null,
    storyStep: 0,
  };
}

function storyView(storyId: string, step: number): AppState["view"] {
  return {
    mode: "story",
    focusedEntityId: null,
    storyId,
    storyStep: Math.max(0, step),
  };
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
          revision: 0,
          loadState: "loading",
          lastError: null,
        },
        view: resetView(),
        ui: { ...state.ui, detailEntityId: null },
        data: { projection: null, detail: null },
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
      if (action.revision <= state.session.revision) {
        return state;
      }
      return {
        ...state,
        session: { ...state.session, revision: action.revision, loadState: "ready" },
        data: { ...state.data, projection: action.projection, detail: null },
      };
    case "themeDerived":
      return {
        ...state,
        // Theme derivation is revision-independent in v0: template changes arrive as a new group load.
        theme: { resolved: action.theme, report: action.report },
      };
    case "entityFocused":
      return {
        ...state,
        view: focusView(action.entityId),
        ui: { ...state.ui, detailEntityId: action.entityId },
      };
    case "entityFocusCleared":
      return { ...state, view: resetView() };
    case "detailReceived":
      if (action.revision < state.session.revision) {
        return state;
      }
      return {
        ...state,
        ui: { ...state.ui, detailEntityId: action.entityId },
        data: { ...state.data, detail: action.detail },
      };
    case "detailCleared":
      return {
        ...state,
        ui: { ...state.ui, detailEntityId: null },
        data: { ...state.data, detail: null },
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
