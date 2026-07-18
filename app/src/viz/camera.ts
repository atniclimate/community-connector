import { PerspectiveCamera, Vector3 } from "three";
import { OrbitControls } from "three/examples/jsm/controls/OrbitControls.js";
import { RENDER_TOKENS } from "./config";

export type CameraMotionSettings = {
  readonly durationMs: number;
  readonly dampingEnabled: boolean;
  readonly motionScale: number;
};

export type CameraRig = {
  readonly camera: PerspectiveCamera;
  readonly controls: OrbitControls;
  readonly flyTo: (position: Vector3, reducedMotion: boolean) => void;
  readonly setDrift: (enabled: boolean) => void;
  readonly setReducedMotion: (reduced: boolean) => void;
  readonly update: (deltaSeconds: number) => boolean;
  readonly dispose: () => void;
};

const MOTION_ON = 1;
const MOTION_OFF = 0;
const ASPECT_FALLBACK = 1;
const MIN_TARGET_LENGTH = 0.0001;
const TARGET_ORIGIN = new Vector3(0, 0, 0);
const CAMERA_START = new Vector3(0, 0, RENDER_TOKENS.camera.initialZ);

export function motionSettings(reducedMotion: boolean): CameraMotionSettings {
  return {
    durationMs: reducedMotion ? MOTION_OFF : RENDER_TOKENS.camera.standardDurationMs,
    dampingEnabled: !reducedMotion,
    motionScale: reducedMotion ? MOTION_OFF : MOTION_ON,
  };
}

/** Fly-to polish (P1.2): short hops get the shorter near duration. */
export function flightDurationMs(distance: number): number {
  return distance <= RENDER_TOKENS.camera.nearFlightDistance
    ? RENDER_TOKENS.camera.nearDurationMs
    : RENDER_TOKENS.camera.standardDurationMs;
}

/**
 * Idle drift policy (P1.2): drift only when enabled by the view mode, the user
 * is not interacting, reduced motion is off, and the idle delay has elapsed.
 */
export function driftActive(
  enabled: boolean,
  interacting: boolean,
  msSinceInteraction: number,
  reducedMotion: boolean,
): boolean {
  return enabled && !interacting && !reducedMotion && msSinceInteraction >= RENDER_TOKENS.drift.idleDelayMs;
}

function easeOutQuad(t: number): number {
  return t * (2 - t);
}

type Flight = {
  readonly fromPosition: Vector3;
  readonly toPosition: Vector3;
  readonly fromTarget: Vector3;
  readonly toTarget: Vector3;
  readonly durationMs: number;
  elapsedMs: number;
};

type RigState = {
  flight: Flight | null;
  driftEnabled: boolean;
  reducedMotion: boolean;
  interacting: boolean;
  msSinceInteraction: number;
};

export function createCameraRig(canvas: HTMLCanvasElement, onViewChange?: () => void): CameraRig {
  const camera = new PerspectiveCamera(
    RENDER_TOKENS.camera.fov,
    ASPECT_FALLBACK,
    RENDER_TOKENS.camera.near,
    RENDER_TOKENS.camera.far,
  );
  camera.position.copy(CAMERA_START);
  const controls = new OrbitControls(camera, canvas);
  controls.enableDamping = true;
  controls.dampingFactor = RENDER_TOKENS.camera.dampingFactor;
  controls.autoRotateSpeed = RENDER_TOKENS.drift.autoRotateSpeed;
  controls.target.copy(TARGET_ORIGIN);
  const state: RigState = {
    flight: null,
    driftEnabled: false,
    reducedMotion: false,
    interacting: false,
    msSinceInteraction: MOTION_OFF,
  };
  const onStart = (): void => {
    state.interacting = true;
    state.msSinceInteraction = MOTION_OFF;
  };
  const onEnd = (): void => {
    state.interacting = false;
    state.msSinceInteraction = MOTION_OFF;
  };
  const onChange = (): void => onViewChange?.();
  controls.addEventListener("start", onStart);
  controls.addEventListener("end", onEnd);
  controls.addEventListener("change", onChange);
  return {
    camera,
    controls,
    flyTo: (position, reducedMotion) => {
      state.msSinceInteraction = MOTION_OFF;
      state.flight = beginFlight(camera, controls, position, reducedMotion);
    },
    setDrift: (enabled) => {
      state.driftEnabled = enabled;
    },
    setReducedMotion: (reduced) => {
      applyReducedMotion(camera, controls, state, reduced);
    },
    update: (deltaSeconds) => updateRig(camera, controls, state, deltaSeconds),
    dispose: () => {
      controls.removeEventListener("start", onStart);
      controls.removeEventListener("end", onEnd);
      controls.removeEventListener("change", onChange);
      controls.dispose();
    },
  };
}

function applyReducedMotion(
  camera: PerspectiveCamera,
  controls: OrbitControls,
  state: RigState,
  reduced: boolean,
): void {
  state.reducedMotion = reduced;
  controls.enableDamping = motionSettings(reduced).dampingEnabled;
  if (reduced && state.flight !== null) {
    // An in-flight animation snaps to its destination; the end state is kept.
    camera.position.copy(state.flight.toPosition);
    controls.target.copy(state.flight.toTarget);
    controls.update();
    state.flight = null;
  }
}

function beginFlight(
  camera: PerspectiveCamera,
  controls: OrbitControls,
  target: Vector3,
  reducedMotion: boolean,
): Flight | null {
  const settings = motionSettings(reducedMotion);
  const direction = target.length() > MIN_TARGET_LENGTH ? target.clone().normalize() : new Vector3(0, 0, 1);
  const toPosition = target.clone().add(direction.multiplyScalar(RENDER_TOKENS.camera.targetDistance));
  controls.enableDamping = settings.dampingEnabled;
  if (settings.durationMs === MOTION_OFF) {
    camera.position.copy(toPosition);
    controls.target.copy(target);
    controls.update();
    return null;
  }
  return {
    fromPosition: camera.position.clone(),
    toPosition,
    fromTarget: controls.target.clone(),
    toTarget: target.clone(),
    durationMs: Math.min(
      flightDurationMs(camera.position.distanceTo(toPosition)),
      RENDER_TOKENS.camera.maxDurationMs,
    ),
    elapsedMs: MOTION_OFF,
  };
}

function advanceFlight(
  camera: PerspectiveCamera,
  controls: OrbitControls,
  state: RigState,
  deltaMs: number,
): void {
  const flight = state.flight;
  if (flight === null) {
    return;
  }
  flight.elapsedMs = Math.min(flight.durationMs, flight.elapsedMs + deltaMs);
  const t = flight.elapsedMs / flight.durationMs;
  camera.position.lerpVectors(flight.fromPosition, flight.toPosition, easeOutQuad(t));
  const targetT = Math.min(MOTION_ON, t * RENDER_TOKENS.camera.aimLockFraction);
  controls.target.lerpVectors(flight.fromTarget, flight.toTarget, easeOutQuad(targetT));
  if (flight.elapsedMs >= flight.durationMs) {
    state.flight = null;
  }
}

function updateRig(
  camera: PerspectiveCamera,
  controls: OrbitControls,
  state: RigState,
  deltaSeconds: number,
): boolean {
  const deltaMs = deltaSeconds * RENDER_TOKENS.time.secondsToMs;
  if (state.flight !== null) {
    advanceFlight(camera, controls, state, deltaMs);
    controls.update();
    return true;
  }
  state.msSinceInteraction += deltaMs;
  controls.autoRotate = driftActive(
    state.driftEnabled,
    state.interacting,
    state.msSinceInteraction,
    state.reducedMotion,
  );
  const needsLoop = controls.autoRotate || controls.enableDamping;
  if (needsLoop) {
    controls.update();
  }
  return needsLoop;
}
