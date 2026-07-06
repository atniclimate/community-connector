import type { Action } from "./actions";
import { reduce } from "./reducer";
import type { AppState, JsonObject, JsonValue } from "./state";

export type Listener = () => void;
export type Reducer = (state: AppState, action: Action) => AppState;

export type Store = {
  readonly getState: () => AppState;
  readonly dispatch: (action: Action) => void;
  readonly subscribe: (listener: Listener) => () => void;
};

type StoreOptions = {
  readonly reducer?: Reducer;
  readonly devFreeze?: boolean;
};

function isObject(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

function deepFreeze<T>(value: T): T {
  if (!isObject(value)) {
    return value;
  }
  Object.freeze(value);
  for (const nested of Object.values(value)) {
    deepFreeze(nested);
  }
  return value;
}

function shouldFreeze(options: StoreOptions): boolean {
  if (options.devFreeze !== undefined) {
    return options.devFreeze;
  }
  return import.meta.env.DEV;
}

/** Implements docs/blueprints/app-state.md store as the only mutation path. */
export function createStore(initial: AppState, options: StoreOptions = {}): Store {
  const reducer = options.reducer ?? reduce;
  const freeze = shouldFreeze(options);
  const listeners = new Set<Listener>();
  let state = freeze ? deepFreeze(initial) : initial;

  return {
    getState: () => state,
    dispatch: (action: Action): void => {
      const nextState = reducer(state, action);
      state = freeze ? deepFreeze(nextState) : nextState;
      for (const listener of listeners) {
        listener();
      }
    },
    subscribe: (listener: Listener): (() => void) => {
      listeners.add(listener);
      return () => {
        listeners.delete(listener);
      };
    },
  };
}
