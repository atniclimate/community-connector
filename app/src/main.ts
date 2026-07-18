import { createInitialState } from "./state/state";
import { createStore } from "./state/store";
import { selectProjectedEntityCount } from "./state/selectors";
import { loadGroup } from "./state/effects";
import { bootSnapshot, readSnapshotEnvelope, snapshotError } from "./state/snapshot";
import { WasmClient } from "./wasm/client";
import { mountViz } from "./viz";
import { mountLegend } from "./viz/legend";
import { mountSearch } from "./ui/search";
import { mountDetailPanel } from "./ui/detail";
import { mountFlatProjection } from "./ui/flat";

declare const __CN_SNAPSHOT_MODE__: boolean;

const app = document.querySelector<HTMLElement>("#app");
if (app === null) {
  throw new Error("Missing #app element");
}
const appElement = app;
const statusElement = document.createElement("div");
const searchElement = document.createElement("div");
const vizElement = document.createElement("div");
const detailElement = document.createElement("div");
const flatElement = document.createElement("div");
statusElement.className = "cn-status";
statusElement.setAttribute("role", "status");
statusElement.setAttribute("aria-live", "polite");
searchElement.className = "cn-search-region";
searchElement.setAttribute("role", "search");
searchElement.setAttribute("aria-label", "Search the current network");
vizElement.className = "cn-viz";
vizElement.setAttribute("role", "region");
vizElement.setAttribute("aria-label", "Network graph and legend");
detailElement.className = "cn-detail-region";
detailElement.setAttribute("role", "complementary");
detailElement.setAttribute("aria-label", "Selected entity details");
flatElement.className = "cn-flat-region";
flatElement.setAttribute("role", "region");
flatElement.setAttribute("aria-label", "Flat network view");
appElement.replaceChildren(
  statusElement,
  searchElement,
  vizElement,
  detailElement,
  flatElement,
);

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
    state.session.lastError === null ? "" : `error: ${state.session.lastError.message}`,
  ].filter((part) => part !== "").join(" | ");
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

const onReducedMotion = (event: MediaQueryListEvent): void => {
  store.dispatch({ kind: "reducedMotionChanged", reducedMotion: event.matches });
};
reducedMotionMedia.addEventListener("change", onReducedMotion);

store.dispatch({ kind: "reducedMotionChanged", reducedMotion: reducedMotionMedia.matches });
const unmounts = [
  store.subscribe(render),
  mountSearch(searchElement, { store, client }),
  mountViz(vizElement, store),
  mountLegend(vizElement, store),
  mountDetailPanel(detailElement, { store, client }),
  mountFlatProjection(flatElement, { store }),
];
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
} else if (__CN_SNAPSHOT_MODE__) {
  try {
    bootSnapshot(store, readSnapshotEnvelope(document));
  } catch (error) {
    store.dispatch({ kind: "errorSurfaced", error: snapshotError(error) });
  }
}

let tornDown = false;
function teardown(): void {
  if (tornDown) {
    return;
  }
  tornDown = true;
  reducedMotionMedia.removeEventListener("change", onReducedMotion);
  for (const unmount of unmounts.reverse()) {
    unmount();
  }
  client.dispose();
  worker.terminate();
}

window.addEventListener("beforeunload", teardown, { once: true });
