import type { AppState } from "./state/state";

declare global {
  interface Window {
    readonly __cn_state_snapshot?: () => AppState;
  }
}

export {};
