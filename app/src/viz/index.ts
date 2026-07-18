import { WebGLRenderer } from "three";
import type { AppState, ProjectionDto } from "../state/state";
import type { Store } from "../state/store";
import { RENDER_TOKENS } from "./config";
import { buildEdgeLayer, type EdgeLayer } from "./edges";
import { buildHaloLayer, type HaloLayer } from "./halos";
import { buildLabelLayer, type LabelLayer } from "./labels";
import { computeLayout, type LayoutResult } from "./layout";
import { buildNodeLayer, degreesForProjection, type NodeLayer } from "./nodes";
import { PickingController } from "./picking";
import { projectedEntities } from "./projection";
import { createVizScene, type SceneSetup } from "./scene";
import { createCameraRig, type CameraRig } from "./camera";
import { QualityManager, type QualityProfile } from "./quality";

export type MountedViz = () => void;

const CANVAS_LABEL_EMPTY = "Community graph loading";
const ZERO = 0;
const UNIT = 1;
const MIN_HEIGHT = 320;

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
  projectionRevision: number | null;
  themeKey: string;
  dirty: boolean;
  frame: number | null;
  lastTime: number | null;
};

export function mountViz(container: HTMLElement, store: Store): MountedViz {
  container.replaceChildren();
  const canvas = document.createElement("canvas");
  const live = document.createElement("div");
  live.setAttribute("aria-live", "polite");
  live.className = "cn-viz-live";
  canvas.setAttribute("role", "img");
  canvas.setAttribute("aria-label", CANVAS_LABEL_EMPTY);
  container.append(canvas, live);
  const renderer = new WebGLRenderer({ canvas, antialias: true });
  const sceneSetup = createVizScene(store.getState().theme.resolved);
  const cameraRig = createCameraRig(canvas);
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
    projectionRevision: null,
    themeKey: "",
    dirty: true,
    frame: null,
    lastTime: null,
  };
  renderState.picking = new PickingController(canvas, cameraRig.camera, () => renderState.nodes, store);
  const unsubscribe = store.subscribe(() => handleState(renderState, container, store));
  const onVisibility = (): void => schedule(renderState, store);
  document.addEventListener("visibilitychange", onVisibility);
  handleState(renderState, container, store);
  return () => {
    unsubscribe();
    document.removeEventListener("visibilitychange", onVisibility);
    disposeRenderState(renderState);
    container.replaceChildren();
  };
}

function handleState(renderState: RenderState, container: HTMLElement, store: Store): void {
  const state = store.getState();
  resize(renderState, container);
  rebuildIfNeeded(renderState, state, () => {
    renderState.dirty = true;
    schedule(renderState, store);
  });
  updateAria(renderState, state);
  renderState.dirty = true;
  schedule(renderState, store);
}

function rebuildIfNeeded(renderState: RenderState, state: AppState, onNeedsRender: () => void): void {
  const projection = state.data.projection;
  const themeKey = JSON.stringify(state.theme.resolved?.tokens ?? {});
  if (projection === null || !needsRebuild(renderState, projection, themeKey)) {
    return;
  }
  disposeGraphLayers(renderState);
  const entities = projectedEntities(projection);
  renderState.layout = computeLayout(entities);
  const degrees = degreesForProjection(projection);
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
  });
  renderState.labels = buildLabelLayer({
    projection,
    layout: renderState.layout,
    kindMeta: state.data.kindMeta,
    theme: state.theme.resolved,
    degrees,
    tier: renderState.profile.tier,
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
}

function needsRebuild(renderState: RenderState, projection: ProjectionDto, themeKey: string): boolean {
  return renderState.projectionRevision !== (projection.revision ?? ZERO) || renderState.themeKey !== themeKey;
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
  const canvas = renderState.renderer.domElement;
  if (canvas.width === width && canvas.height === height) {
    return;
  }
  renderState.cameraRig.camera.aspect = width / height;
  renderState.cameraRig.camera.updateProjectionMatrix();
  renderState.renderer.setSize(width, height, false);
  renderState.renderer.setPixelRatio(renderState.profile.dpr);
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
  const previousTier = renderState.profile.tier;
  renderState.profile = renderState.quality.sample(deltaSeconds * RENDER_TOKENS.time.secondsToMs);
  if (renderState.profile.tier !== previousTier) {
    renderState.projectionRevision = null;
    store.dispatch({ kind: "qualityTierChanged", tier: renderState.profile.tier });
  }
  renderState.renderer.setPixelRatio(renderState.profile.dpr);
  const animated = renderState.cameraRig.update(deltaSeconds);
  const labelsChanged =
    renderState.labels?.update(renderState.cameraRig.camera, deltaSeconds * RENDER_TOKENS.time.secondsToMs) ?? false;
  renderState.dirty = renderState.dirty || labelsChanged;
  if (renderState.dirty || animated) {
    renderState.renderer.render(renderState.sceneSetup.scene, renderState.cameraRig.camera);
    renderState.dirty = false;
  }
  if (animated && !state.ui.reducedMotion) {
    schedule(renderState, store);
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
