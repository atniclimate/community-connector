import { createInitialState } from "./state/state";
import { createStore } from "./state/store";
import { selectProjectedEntityCount } from "./state/selectors";
import { loadGroup } from "./state/effects";
import { WasmClient } from "./wasm/client";
import { mountViz } from "./viz";

const app = document.querySelector<HTMLElement>("#app");
if (app === null) {
  throw new Error("Missing #app element");
}
const appElement = app;
const statusElement = document.createElement("div");
const vizElement = document.createElement("div");
statusElement.className = "cn-status";
vizElement.className = "cn-viz";
appElement.replaceChildren(statusElement, vizElement);

const reducedMotionMedia = matchMedia("(prefers-reduced-motion: reduce)");
const store = createStore(createInitialState());
const worker = new Worker(new URL("./wasm/worker.ts", import.meta.url), { type: "module" });
const client = new WasmClient(worker);
const DEMO_GROUP_ID = "00000000-0000-0000-0000-000000000010";
const DEMO_VIEWER_PERSON = "00000000-0000-0000-0000-0000000003e9";

function render(): void {
  const state = store.getState();
  statusElement.textContent = [
    `Community Navigator`,
    `load: ${state.session.loadState}`,
    `quality: ${state.ui.qualityTier}`,
    `entities: ${selectProjectedEntityCount(state)}`,
  ].join(" | ");
}

async function loadDevDemo(): Promise<void> {
  const [templateResponse, opsResponse] = await Promise.all([
    fetch("/fixtures/templates/research-network.template.json"),
    fetch("/fixtures/groups/research-network.ops.jsonl"),
  ]);
  if (!templateResponse.ok || !opsResponse.ok) {
    throw new Error("Failed to fetch dev demo fixtures");
  }
  await loadGroup(
    store,
    client,
    DEMO_GROUP_ID,
    { kind: "person", person: DEMO_VIEWER_PERSON },
    await templateResponse.text(),
    await opsResponse.text(),
  );
}

reducedMotionMedia.addEventListener("change", (event) => {
  store.dispatch({ kind: "reducedMotionChanged", reducedMotion: event.matches });
});

store.dispatch({ kind: "reducedMotionChanged", reducedMotion: reducedMotionMedia.matches });
store.subscribe(render);
mountViz(vizElement, store);
render();

if (import.meta.env.DEV) {
  Object.defineProperty(window, "__cn_state_snapshot", {
    value: () => store.getState(),
    configurable: false,
    enumerable: false,
    writable: false,
  });
  loadDevDemo().catch((error: unknown) => {
    store.dispatch({
      kind: "errorSurfaced",
      error: client.toErrorEnvelope(error),
    });
  });
}

void client;
