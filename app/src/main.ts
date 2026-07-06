import { createInitialState } from "./state/state";
import { createStore } from "./state/store";
import { selectProjectedEntityCount } from "./state/selectors";
import { WasmClient } from "./wasm/client";

const app = document.querySelector<HTMLElement>("#app");
if (app === null) {
  throw new Error("Missing #app element");
}
const appElement = app;

const reducedMotionMedia = matchMedia("(prefers-reduced-motion: reduce)");
const store = createStore(createInitialState());
const worker = new Worker(new URL("./wasm/worker.ts", import.meta.url), { type: "module" });
const client = new WasmClient(worker);

function render(): void {
  const state = store.getState();
  appElement.textContent = [
    `Community Navigator`,
    `load: ${state.session.loadState}`,
    `entities: ${selectProjectedEntityCount(state)}`,
  ].join(" | ");
}

reducedMotionMedia.addEventListener("change", (event) => {
  store.dispatch({ kind: "reducedMotionChanged", reducedMotion: event.matches });
});

store.dispatch({ kind: "reducedMotionChanged", reducedMotion: reducedMotionMedia.matches });
store.subscribe(render);
render();

if (import.meta.env.DEV) {
  Object.defineProperty(window, "__cn_state_snapshot", {
    value: () => store.getState(),
    configurable: false,
    enumerable: false,
    writable: false,
  });
}

void client;
