import { NeutralToneMapping, SRGBColorSpace, Vector2, WebGLRenderer } from "three";
import type { AppState, PresentBeat, ProjectionDto, ProjectionEntityDto, ViewMode } from "../state/state";
import type { Store } from "../state/store";
import type { WasmClient } from "../wasm/client";
import { RENDER_TOKENS } from "./config";
import { buildEdgeLayer, setEdgeFocusBlend, writeFocusTargetColors, type EdgeLayer } from "./edges";
import { buildHaloLayer, setHaloFocusDim, type HaloLayer } from "./halos";
import { buildLabelLayer, type LabelLayer } from "./labels";
import { computeLayout, type LayoutResult } from "./layout";
import { buildNodeLayer, degreesForProjection, type NodeLayer } from "./nodes";
import { PickingController } from "./picking";
import { entityLabel, projectedEntities } from "./projection";
import { createVizScene, type SceneSetup } from "./scene";
import { createCameraRig, zoomToFit, type CameraRig } from "./camera";
import { applyFocusToNodeLayer, computeFocusSet, FocusBlend, writeNodeHover, type FocusSet } from "./focus";
import { measureHighlights, presenterBeatText } from "./presenter";
import { effectivePixelRatio, QualityManager, type QualityProfile } from "./quality";

export type MountedViz = () => void;

const CANVAS_LABEL_EMPTY = "Community graph loading";
const ZERO = 0;
const UNIT = 1;
const MIN_HEIGHT = 320;
const BLEND_OFF = 0;
const BLEND_ON = 1;
const LOGICAL_SIZE = new Vector2();
const TOOLTIP_OFFSET = 14;
const TOOLTIP_MARGIN = 8;

type RenderState = {
  sceneSetup: SceneSetup;
  cameraRig: CameraRig;
  renderer: WebGLRenderer;
  layout: LayoutResult | null;
  nodes: NodeLayer | null;
  edges: EdgeLayer | null;
  halos: HaloLayer | null;
  labels: LabelLayer | null;
  picking: PickingController | null;
  quality: QualityManager;
  profile: QualityProfile;
  degrees: ReadonlyMap<string, number> | null;
  focusedEntityId: string | null;
  focus: FocusSet | null;
  hoveredEntityId: string | null;
  entities: ReadonlyMap<string, ProjectionEntityDto>;
  tooltip: HTMLDivElement;
  pointer: { x: number; y: number };
  focusBlend: FocusBlend;
  projectionRevision: number | null;
  fittedLoadKey: string | null;
  themeKey: string;
  viewMode: ViewMode;
  presentBeatIndex: number | null;
  presentMeasureKey: string | null;
  highlightedIds: ReadonlySet<string>;
  measureExplanations: readonly string[];
  live: HTMLDivElement;
  beatLabel: HTMLDivElement;
  dirty: boolean;
  frame: number | null;
  lastTime: number | null;
};

export function mountViz(container: HTMLElement, store: Store, client: WasmClient): MountedViz {
  container.replaceChildren();
  const canvas = document.createElement("canvas");
  const live = document.createElement("div");
  const beatLabel = document.createElement("div");
  live.setAttribute("aria-live", "polite");
  live.className = "cn-viz-live";
  beatLabel.className = "cn-present-beat";
  beatLabel.hidden = true;
  const tooltip = document.createElement("div");
  tooltip.className = "cn-viz-tooltip";
  // Visual echo of the hovered node; the flat view and search are the
  // accessible paths to the same names (I9).
  tooltip.setAttribute("aria-hidden", "true");
  tooltip.hidden = true;
  canvas.setAttribute("role", "img");
  canvas.setAttribute("aria-label", CANVAS_LABEL_EMPTY);
  canvas.setAttribute("tabindex", "0");
  container.append(canvas, live, beatLabel, tooltip);
  const renderer = new WebGLRenderer({ canvas, antialias: true });
  renderer.outputColorSpace = SRGBColorSpace;
  renderer.toneMapping = NeutralToneMapping;
  const sceneSetup = createVizScene(store.getState().theme.resolved);
  let markViewDirty = (): void => {
    // Replaced below once renderState exists; camera changes before then are
    // covered by the initial handleState render.
  };
  const cameraRig = createCameraRig(canvas, () => markViewDirty());
  const quality = new QualityManager();
  const renderState: RenderState = {
    sceneSetup,
    cameraRig,
    renderer,
    layout: null,
    nodes: null,
    edges: null,
    halos: null,
    labels: null,
    picking: null,
    quality,
    profile: quality.profile,
    degrees: null,
    focusedEntityId: null,
    focus: null,
    hoveredEntityId: null,
    entities: new Map(),
    tooltip,
    pointer: { x: ZERO, y: ZERO },
    focusBlend: new FocusBlend(),
    projectionRevision: null,
    fittedLoadKey: null,
    themeKey: "",
    viewMode: "overview",
    presentBeatIndex: null,
    presentMeasureKey: null,
    highlightedIds: new Set<string>(),
    measureExplanations: [],
    live,
    beatLabel,
    dirty: true,
    frame: null,
    lastTime: null,
  };
  markViewDirty = () => {
    renderState.dirty = true;
    schedule(renderState, store);
  };
  renderState.picking = new PickingController(canvas, cameraRig.camera, () => renderState.nodes, store);
  const unsubscribe = store.subscribe(() => handleState(renderState, container, store, client));
  const onVisibility = (): void => {
    // A hidden interval must not reach the frame clock or the quality EMA.
    renderState.lastTime = null;
    schedule(renderState, store);
  };
  const onKeydown = (event: KeyboardEvent): void => handlePresenterKeydown(event, renderState, store);
  const onPointerMove = (event: PointerEvent): void => {
    const rect = container.getBoundingClientRect();
    renderState.pointer.x = event.clientX - rect.left;
    renderState.pointer.y = event.clientY - rect.top;
    if (!tooltip.hidden) {
      positionTooltip(renderState, container);
    }
  };
  canvas.addEventListener("pointermove", onPointerMove);
  const resizeObserver = new ResizeObserver(() => {
    resize(renderState, container);
    renderState.dirty = true;
    schedule(renderState, store);
  });
  resizeObserver.observe(container);
  document.addEventListener("visibilitychange", onVisibility);
  window.addEventListener("keydown", onKeydown);
  handleState(renderState, container, store, client);
  return () => {
    resizeObserver.disconnect();
    canvas.removeEventListener("pointermove", onPointerMove);
    unsubscribe();
    document.removeEventListener("visibilitychange", onVisibility);
    window.removeEventListener("keydown", onKeydown);
    disposeRenderState(renderState);
    container.replaceChildren();
  };
}

function handleState(
  renderState: RenderState,
  container: HTMLElement,
  store: Store,
  client: WasmClient,
): void {
  const state = store.getState();
  resize(renderState, container);
  const rebuilt = rebuildIfNeeded(renderState, state, () => {
    renderState.dirty = true;
    schedule(renderState, store);
  });
  syncMotion(renderState, state, rebuilt, store, client);
  syncHover(renderState, state, container);
  updateAria(renderState, state);
  renderState.dirty = true;
  schedule(renderState, store);
}

function rebuildIfNeeded(renderState: RenderState, state: AppState, onNeedsRender: () => void): boolean {
  const projection = state.data.projection;
  const themeKey = JSON.stringify(state.theme.resolved?.tokens ?? {});
  if (projection === null || !needsRebuild(renderState, projection, themeKey, state.view.mode)) {
    return false;
  }
  disposeGraphLayers(renderState);
  const entities = projectedEntities(projection);
  renderState.entities = new Map((projection.entities ?? []).map((entity) => [entity.id, entity]));
  renderState.layout = computeLayout(entities);
  const degrees = degreesForProjection(projection);
  renderState.degrees = degrees;
  renderState.nodes = buildNodeLayer({
    projection,
    layout: renderState.layout,
    kindMeta: state.data.kindMeta,
    theme: state.theme.resolved,
    degrees,
  });
  renderState.edges = buildEdgeLayer(projection, renderState.layout, state.theme.resolved);
  renderState.halos = buildHaloLayer({
    projection,
    layout: renderState.layout,
    kindMeta: state.data.kindMeta,
    theme: state.theme.resolved,
    tier: renderState.profile.tier,
    cameraPosition: renderState.cameraRig.camera.position.clone(),
    viewMode: state.view.mode,
  });
  renderState.labels = buildLabelLayer({
    projection,
    layout: renderState.layout,
    kindMeta: state.data.kindMeta,
    theme: state.theme.resolved,
    degrees,
    tier: renderState.profile.tier,
    viewMode: state.view.mode,
    onNeedsRender,
  });
  renderState.sceneSetup.scene.add(
    renderState.edges.object,
    renderState.nodes.group,
    renderState.halos.group,
    renderState.labels.group,
  );
  renderState.projectionRevision = projection.revision ?? ZERO;
  renderState.themeKey = themeKey;
  renderState.viewMode = state.view.mode;
  return true;
}

/**
 * P1.2 focus/motion wiring. Focus state comes exclusively from the app state
 * machine (view.mode / view.focusedEntityId); this only projects it onto the
 * already-built render layers, and re-projects after any layer rebuild.
 */
function syncMotion(
  renderState: RenderState,
  state: AppState,
  rebuilt: boolean,
  store: Store,
  client: WasmClient,
): void {
  const reduced = state.ui.reducedMotion;
  const present = state.view.mode === "present";
  renderState.cameraRig.setReducedMotion(reduced);
  renderState.cameraRig.setDrift(
    (state.view.mode === "overview" || present) && !reduced,
    present ? RENDER_TOKENS.drift.presentAutoRotateSpeed : RENDER_TOKENS.drift.autoRotateSpeed,
  );
  renderState.sceneSetup.setPresentMode(present);
  const beat = present ? state.presentation.beats[state.presentation.beatIndex] : undefined;
  syncPresenterMeasure(renderState, state, beat, store, client);
  updatePresenterLabel(renderState, present, beat);
  fitNewLoad(renderState, state, rebuilt);
  const focusedId = state.view.mode === "focus" || present ? state.view.focusedEntityId : null;
  const focusChanged = focusedId !== renderState.focusedEntityId;
  const presentBeatIndex = present ? state.presentation.beatIndex : null;
  const presentBeatChanged = presentBeatIndex !== renderState.presentBeatIndex;
  if (!focusChanged && !presentBeatChanged && !rebuilt) {
    return;
  }
  renderState.focusedEntityId = focusedId;
  renderState.presentBeatIndex = presentBeatIndex;
  const projection = state.data.projection;
  if (projection === null || renderState.nodes === null || renderState.edges === null || renderState.degrees === null) {
    return;
  }
  const focus = applyFocusRendering(renderState, state, focusedId);
  if (focusChanged && focusedId !== null) {
    const position = renderState.layout?.positions.get(focusedId);
    if (position !== undefined) {
      renderState.cameraRig.flyTo(position.clone(), reduced);
    }
  } else if (present && (presentBeatChanged || rebuilt) && renderState.layout !== null) {
    zoomToFit(renderState.cameraRig, positionsForBeat(projection, renderState.layout, beat), {
      paddingWorldUnits: RENDER_TOKENS.camera.fitAllPaddingWorldUnits,
      reducedMotion: reduced,
    });
  }
}

/**
 * Frames each newly loaded group once. Theme, tier, and view rebuilds keep
 * the user's camera. It snaps rather than flies: there is no prior view of
 * this data to animate from.
 */
function fitNewLoad(renderState: RenderState, state: AppState, rebuilt: boolean): void {
  const projection = state.data.projection;
  const loadKey = `${state.session.groupId ?? ""}:${state.session.sessionId}`;
  if (!rebuilt || projection === null || renderState.layout === null || renderState.fittedLoadKey === loadKey) {
    return;
  }
  renderState.fittedLoadKey = loadKey;
  if (state.view.mode !== "overview") {
    return;
  }
  zoomToFit(renderState.cameraRig, positionsForBeat(projection, renderState.layout, undefined), {
    paddingWorldUnits: RENDER_TOKENS.camera.fitAllPaddingWorldUnits,
    reducedMotion: true,
    bearing: "current",
  });
}

function applyFocusRendering(
  renderState: RenderState,
  state: AppState,
  focusedId: string | null,
): ReturnType<typeof computeFocusSet> {
  const projection = state.data.projection;
  if (projection === null || renderState.nodes === null || renderState.edges === null || renderState.degrees === null) {
    return null;
  }
  const focus = computeFocusSet(projection, focusedId, renderState.highlightedIds);
  renderState.focus = focus;
  applyFocusToNodeLayer(renderState.nodes, projection, state.theme.resolved, renderState.degrees, focus);
  // The role pass rewrote every instance; put the hover emphasis back on top.
  writeHover(renderState, state, renderState.hoveredEntityId, true);
  writeFocusTargetColors(renderState.edges, state.theme.resolved, focus?.adjacentEdgeIds ?? null);
  renderState.halos?.setSelected(focusedId);
  renderState.focusBlend.setTarget(focus === null ? BLEND_OFF : BLEND_ON);
  return focus;
}

function writeHover(renderState: RenderState, state: AppState, entityId: string | null, hovered: boolean): void {
  const entity = entityId === null ? undefined : renderState.entities.get(entityId);
  if (entityId === null || entity === undefined || renderState.nodes === null || renderState.degrees === null) {
    return;
  }
  writeNodeHover(renderState.nodes, entityId, {
    kind: entity.kind ?? "",
    degree: renderState.degrees.get(entityId) ?? RENDER_TOKENS.node.minDegree,
    focus: renderState.focus,
    theme: state.theme.resolved,
  }, hovered);
}

function syncHover(renderState: RenderState, state: AppState, container: HTMLElement): void {
  const next = state.view.hoveredEntityId;
  if (next === renderState.hoveredEntityId) {
    return;
  }
  writeHover(renderState, state, renderState.hoveredEntityId, false);
  renderState.hoveredEntityId = next;
  writeHover(renderState, state, next, true);
  const entity = next === null ? undefined : renderState.entities.get(next);
  const tooltip = renderState.tooltip;
  if (entity === undefined) {
    tooltip.hidden = true;
    return;
  }
  const name = document.createElement("strong");
  name.textContent = entityLabel(entity, state.data.kindMeta);
  const kind = document.createElement("span");
  kind.textContent = state.data.kindMeta[entity.kind ?? ""]?.label ?? entity.kind ?? "";
  tooltip.replaceChildren(name, kind);
  tooltip.hidden = false;
  positionTooltip(renderState, container);
}

function positionTooltip(renderState: RenderState, container: HTMLElement): void {
  const tooltip = renderState.tooltip;
  const maxX = Math.max(ZERO, container.clientWidth - tooltip.offsetWidth - TOOLTIP_MARGIN);
  const maxY = Math.max(ZERO, container.clientHeight - tooltip.offsetHeight - TOOLTIP_MARGIN);
  const x = Math.min(maxX, renderState.pointer.x + TOOLTIP_OFFSET);
  const y = Math.min(maxY, renderState.pointer.y + TOOLTIP_OFFSET);
  tooltip.style.transform = `translate(${Math.max(TOOLTIP_MARGIN, x)}px, ${Math.max(TOOLTIP_MARGIN, y)}px)`;
}

function activeMeasureKey(state: AppState, beat: PresentBeat | undefined): string | null {
  if (state.view.mode !== "present" || state.session.groupId === null || beat?.measure === undefined) {
    return null;
  }
  return [
    state.session.sessionId,
    state.session.revision,
    state.presentation.beatIndex,
    beat.id,
    beat.measure,
    beat.topN ?? "",
  ].join(":");
}

function syncPresenterMeasure(
  renderState: RenderState,
  state: AppState,
  beat: PresentBeat | undefined,
  store: Store,
  client: WasmClient,
): void {
  const key = activeMeasureKey(state, beat);
  if (key === renderState.presentMeasureKey) {
    return;
  }
  renderState.presentMeasureKey = key;
  renderState.highlightedIds = new Set<string>();
  renderState.measureExplanations = [];
  if (key !== null && beat?.measure !== undefined && state.session.groupId !== null) {
    void loadPresenterMeasure(renderState, store, client, state, beat, key);
  }
}

async function loadPresenterMeasure(
  renderState: RenderState,
  store: Store,
  client: WasmClient,
  requestState: AppState,
  beat: PresentBeat,
  key: string,
): Promise<void> {
  const groupId = requestState.session.groupId;
  if (groupId === null) {
    return;
  }
  try {
    const response = await client.graphMeasures(groupId, requestState.session.viewer, {
      membership_kind: "member_of",
      jaccard_pairs: [],
    });
    const current = store.getState();
    if (renderState.presentMeasureKey !== key || activeMeasureKey(current, beat) !== key) {
      return;
    }
    const highlights = measureHighlights(response, beat);
    renderState.highlightedIds = highlights.ids;
    renderState.measureExplanations = highlights.explanations;
    updatePresenterLabel(renderState, true, beat);
    applyFocusRendering(renderState, current, current.view.focusedEntityId);
    renderState.dirty = true;
    schedule(renderState, store);
  } catch (error) {
    const current = store.getState();
    if (renderState.presentMeasureKey === key && activeMeasureKey(current, beat) === key) {
      store.dispatch({ kind: "errorSurfaced", error: client.toErrorEnvelope(error) });
    }
  }
}

function updatePresenterLabel(
  renderState: RenderState,
  present: boolean,
  beat: PresentBeat | undefined,
): void {
  const text = present ? presenterBeatText(beat, renderState.measureExplanations) : "";
  renderState.beatLabel.hidden = !present || text === "";
  renderState.beatLabel.textContent = text;
  renderState.live.textContent = text;
}

function positionsForBeat(
  projection: ProjectionDto,
  layout: LayoutResult,
  beat: PresentBeat | undefined,
): readonly import("three").Vector3[] {
  const kinds = beat?.filter?.kinds;
  return projectedEntities(projection).flatMap((entity) => {
    if (kinds !== undefined && !kinds.includes(entity.kind ?? "")) {
      return [];
    }
    const position = layout.positions.get(entity.id);
    return position === undefined ? [] : [position];
  });
}

function handlePresenterKeydown(event: KeyboardEvent, renderState: RenderState, store: Store): void {
  const state = store.getState();
  if (state.view.mode !== "present") {
    return;
  }
  const lastBeatIndex = Math.max(ZERO, state.presentation.beats.length - UNIT);
  let beatIndex: number | null = null;
  if (event.key === " " || event.code === "Space" || event.key === "ArrowRight") {
    beatIndex = Math.min(lastBeatIndex, state.presentation.beatIndex + UNIT);
  } else if (event.key === "ArrowLeft") {
    beatIndex = Math.max(ZERO, state.presentation.beatIndex - UNIT);
  } else if (event.key === "Home") {
    beatIndex = ZERO;
  } else if (event.key.toLowerCase() === "f") {
    const projection = state.data.projection;
    if (projection !== null && renderState.layout !== null) {
      zoomToFit(renderState.cameraRig, positionsForBeat(projection, renderState.layout, undefined), {
        paddingWorldUnits: RENDER_TOKENS.camera.fitAllPaddingWorldUnits,
        reducedMotion: state.ui.reducedMotion,
      });
    }
  } else if (event.key === "Escape") {
    store.dispatch({ kind: "presentExited" });
  } else {
    return;
  }
  event.preventDefault();
  if (beatIndex !== null) {
    store.dispatch({ kind: "presentBeatAdvanced", beatIndex });
  }
}

function needsRebuild(
  renderState: RenderState,
  projection: ProjectionDto,
  themeKey: string,
  viewMode: ViewMode,
): boolean {
  return renderState.projectionRevision !== (projection.revision ?? ZERO)
    || renderState.themeKey !== themeKey
    || renderState.viewMode !== viewMode;
}

function updateAria(renderState: RenderState, state: AppState): void {
  const count = state.data.projection?.entities?.length ?? ZERO;
  const selected = state.view.focusedEntityId;
  const label = selected === null
    ? `Community graph with ${count} entities`
    : `Community graph with ${count} entities. Selected entity ${selected}`;
  renderState.renderer.domElement.setAttribute("aria-label", label);
}

function resize(renderState: RenderState, container: HTMLElement): void {
  const width = Math.max(UNIT, container.clientWidth);
  const height = Math.max(MIN_HEIGHT, container.clientHeight);
  const renderer = renderState.renderer;
  const pixelRatio = effectivePixelRatio(window.devicePixelRatio, renderState.profile.dpr);
  if (renderer.getPixelRatio() !== pixelRatio) {
    renderer.setPixelRatio(pixelRatio);
  }
  renderer.getSize(LOGICAL_SIZE);
  if (LOGICAL_SIZE.x === width && LOGICAL_SIZE.y === height) {
    return;
  }
  renderState.cameraRig.camera.aspect = width / height;
  renderState.cameraRig.camera.updateProjectionMatrix();
  renderer.setSize(width, height, false);
}

function schedule(renderState: RenderState, store: Store): void {
  if (document.hidden || renderState.frame !== null) {
    return;
  }
  renderState.frame = requestAnimationFrame((time) => frame(renderState, store, time));
}

function frame(renderState: RenderState, store: Store, time: number): void {
  renderState.frame = null;
  const state = store.getState();
  const deltaSeconds = renderState.lastTime === null ? ZERO : (time - renderState.lastTime) / RENDER_TOKENS.time.secondsToMs;
  renderState.lastTime = time;
  const deltaMs = deltaSeconds * RENDER_TOKENS.time.secondsToMs;
  if (deltaMs > ZERO) {
    const previousTier = renderState.profile.tier;
    renderState.profile = renderState.quality.sample(deltaMs);
    if (renderState.profile.tier !== previousTier) {
      renderState.projectionRevision = null;
      // The store subscriber re-runs resize(), which applies the new DPR cap.
      store.dispatch({ kind: "qualityTierChanged", tier: renderState.profile.tier });
    }
  }
  const animated = renderState.cameraRig.update(deltaSeconds);
  const labelsChanged = renderState.labels?.update(renderState.cameraRig.camera, deltaMs) ?? false;
  const halos = renderState.halos;
  const cameraPosition = renderState.cameraRig.camera.position;
  const halosMoved = halos !== null
    && halos.lastRefreshPosition.distanceTo(cameraPosition) > RENDER_TOKENS.halo.refreshDistance;
  if (halosMoved) {
    halos.refresh(cameraPosition);
  }
  const blendChanged = renderState.focusBlend.update(deltaMs, state.ui.reducedMotion);
  if (renderState.edges !== null) {
    setEdgeFocusBlend(renderState.edges, renderState.focusBlend.value);
  }
  if (renderState.halos !== null) {
    setHaloFocusDim(renderState.halos, renderState.focusBlend.value);
  }
  renderState.dirty = renderState.dirty || labelsChanged || blendChanged || halosMoved;
  if (renderState.dirty || animated) {
    renderState.renderer.render(renderState.sceneSetup.scene, renderState.cameraRig.camera);
    renderState.dirty = false;
  }
  if ((animated || blendChanged) && !state.ui.reducedMotion) {
    schedule(renderState, store);
  }
  if (renderState.frame === null) {
    // Loop is idle; the next wake-up starts a fresh clock instead of reporting
    // the whole idle gap as one slow frame.
    renderState.lastTime = null;
  }
}

function disposeGraphLayers(renderState: RenderState): void {
  if (renderState.nodes !== null) {
    renderState.sceneSetup.scene.remove(renderState.nodes.group);
    renderState.nodes.dispose();
  }
  if (renderState.edges !== null) {
    renderState.sceneSetup.scene.remove(renderState.edges.object);
    renderState.edges.dispose();
  }
  if (renderState.halos !== null) {
    renderState.sceneSetup.scene.remove(renderState.halos.group);
    renderState.halos.dispose();
  }
  if (renderState.labels !== null) {
    renderState.sceneSetup.scene.remove(renderState.labels.group);
    renderState.labels.dispose();
    renderState.labels = null;
  }
}

function disposeRenderState(renderState: RenderState): void {
  if (renderState.frame !== null) {
    cancelAnimationFrame(renderState.frame);
  }
  renderState.picking?.dispose();
  disposeGraphLayers(renderState);
  renderState.cameraRig.dispose();
  renderState.sceneSetup.dispose();
  renderState.renderer.dispose();
}
