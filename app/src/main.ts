// Presenter-mode caption face (D-101/CS-05): self-hosted via Vite, weight 600
// only, latin + latin-ext + vietnamese subsets, zero runtime font requests.
import "@fontsource/league-spartan/600.css";
import { createInitialState, type JsonObject } from "./state/state";
import { BeatSheetError, validateBeats } from "./state/beats";
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
import { mountIntakeWizard } from "./ui/intake/panel";

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
const intakeElement = document.createElement("div");
const statusText = document.createElement("span");
const presentButton = document.createElement("button");
statusElement.className = "cn-status";
statusElement.setAttribute("role", "toolbar");
statusElement.setAttribute("aria-label", "Application controls");
statusText.setAttribute("role", "status");
statusText.setAttribute("aria-live", "polite");
presentButton.className = "cn-present-enter";
presentButton.type = "button";
presentButton.textContent = "Present";
presentButton.setAttribute("aria-label", "Enter presenter mode");
statusElement.append(statusText, presentButton);
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
intakeElement.className = "cn-intake-region";
appElement.replaceChildren(
  statusElement,
  searchElement,
  vizElement,
  detailElement,
  flatElement,
  intakeElement,
);

const reducedMotionMedia = matchMedia("(prefers-reduced-motion: reduce)");
const store = createStore(createInitialState());
const worker = new Worker(new URL("./wasm/worker.ts", import.meta.url), { type: "module" });
const client = new WasmClient(worker);
type DevGroup = {
  readonly fixture: string;
  readonly groupId: string;
  /** A synthetic person holding the governance role in that fixture. */
  readonly viewerPerson: string;
};

// Synthetic fixtures only (I1). Selected with ?group=<key>; research-network
// stays the default so existing dev and test flows are unchanged.
const DEV_GROUPS: Readonly<Record<string, DevGroup>> = {
  "research-network": {
    fixture: "research-network",
    groupId: "00000000-0000-0000-0000-000000000010",
    viewerPerson: "00000000-0000-0000-0000-0000000003e9",
  },
  "atni-convention": {
    fixture: "atni-convention",
    groupId: "00000000-0000-0000-0000-0000000dbba0",
    viewerPerson: "00000000-0000-0000-0000-0000000de2bd",
  },
};
const DEFAULT_DEV_GROUP = "research-network";

function render(): void {
  const state = store.getState();
  appElement.dataset.viewMode = state.view.mode;
  statusText.textContent = [
    `Community Navigator`,
    `load: ${state.session.loadState}`,
    `quality: ${state.ui.qualityTier}`,
    `entities: ${selectProjectedEntityCount(state)}`,
    state.session.lastError === null ? "" : `error: ${state.session.lastError.message}`,
  ].filter((part) => part !== "").join(" | ");
  presentButton.disabled = state.presentation.loadState !== "ready" || state.presentation.beats.length === 0;
}

const onPresent = (): void => store.dispatch({ kind: "presentEntered", beatIndex: 0 });
presentButton.addEventListener("click", onPresent);

async function loadPresentBeats(): Promise<void> {
  try {
    const response = await fetch("/beats.atni.json");
    if (!response.ok) {
      throw new Error(`Failed to fetch presenter beats (${response.status})`);
    }
    // Validated, never cast: a malformed sheet (or one that names people on
    // stage, D-099) takes the same error path as a failed fetch.
    const beats = validateBeats(await response.json());
    store.dispatch({ kind: "presentBeatsLoaded", beats });
  } catch (error) {
    const envelope = error instanceof BeatSheetError ? error.envelope : client.toErrorEnvelope(error);
    store.dispatch({ kind: "errorSurfaced", error: envelope });
  }
}

// Template holder for the intake wizard's form renderer (set at load).
let loadedTemplate: JsonObject | null = null;
let unmountIntake: (() => void) | null = null;

/** Mounts the wizard only when the loaded group resolves the viewer to
 * facilitator-or-governance - an affordance; the core enforces authority
 * regardless (blueprint section 5). Parameterized on the ACTIVE load
 * (round-1 F11), not demo constants; every interactive load path calls
 * this after commit, remounting on viewer/group change. The snapshot
 * build never mounts it (read-only artifact with no worker). */
async function mountIntakeIfFacilitator(
  groupId: string,
  viewer: { readonly kind: string },
): Promise<void> {
  const roles = await client.viewerRoles(groupId, viewer);
  const names = Array.isArray(roles["roles"]) ? roles["roles"] : [];
  const allowed = names.some((role) => role === "facilitator" || role === "governance");
  if (unmountIntake !== null) {
    unmountIntake();
    unmountIntake = null;
  }
  if (!allowed) {
    return;
  }
  unmountIntake = mountIntakeWizard(intakeElement, {
    store,
    client,
    groupId: () => store.getState().session.groupId,
    viewer: () => store.getState().session.viewer,
    template: () => loadedTemplate,
  });
}

function selectedDevGroup(): DevGroup {
  const key = new URLSearchParams(window.location.search).get("group") ?? DEFAULT_DEV_GROUP;
  const group = DEV_GROUPS[key];
  if (group === undefined) {
    throw new Error(`Unknown dev group "${key}"; expected one of: ${Object.keys(DEV_GROUPS).join(", ")}`);
  }
  return group;
}

async function loadDevDemo(): Promise<void> {
  const group = selectedDevGroup();
  const [templateResponse, opsResponse] = await Promise.all([
    fetch(`/fixtures/templates/${group.fixture}.template.json`),
    fetch(`/fixtures/groups/${group.fixture}.ops.jsonl`),
  ]);
  if (!templateResponse.ok || !opsResponse.ok) {
    throw new Error(`Failed to fetch dev fixtures for ${group.fixture}`);
  }
  const templateText = await templateResponse.text();
  const viewer = { kind: "person", person: group.viewerPerson };
  await loadGroup(store, client, group.groupId, viewer, templateText, await opsResponse.text());
  loadedTemplate = JSON.parse(templateText) as JsonObject;
  await mountIntakeIfFacilitator(group.groupId, viewer);
}

const onReducedMotion = (event: MediaQueryListEvent): void => {
  store.dispatch({ kind: "reducedMotionChanged", reducedMotion: event.matches });
};
reducedMotionMedia.addEventListener("change", onReducedMotion);

store.dispatch({ kind: "reducedMotionChanged", reducedMotion: reducedMotionMedia.matches });
const unmounts = [
  store.subscribe(render),
  mountSearch(searchElement, { store, client }),
  mountViz(vizElement, store, client),
  mountLegend(vizElement, store),
  mountDetailPanel(detailElement, { store, client }),
  mountFlatProjection(flatElement, { store }),
];
render();
void loadPresentBeats();

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
  presentButton.removeEventListener("click", onPresent);
  for (const unmount of unmounts.reverse()) {
    unmount();
  }
  if (unmountIntake !== null) {
    unmountIntake();
  }
  client.dispose();
  worker.terminate();
}

window.addEventListener("beforeunload", teardown, { once: true });
