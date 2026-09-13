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
  readonly zoomToFit: (positions: readonly Vector3[], opts: ZoomToFitOptions) => void;
  readonly setDrift: (enabled: boolean, autoRotateSpeed?: number) => void;
  readonly setReducedMotion: (reduced: boolean) => void;
  readonly update: (deltaSeconds: number) => boolean;
  readonly dispose: () => void;
};

export type ZoomToFitOptions = {
  readonly paddingWorldUnits: number;
  readonly reducedMotion: boolean;
  /**
   * "centroid" (presenter beats, blueprint rule): look in along the
   * origin-to-centroid direction. "current": keep the present viewing bearing.
   */
  readonly bearing?: "centroid" | "current";
  /** Fraction of the viewport height at the bottom to keep clear (0..1). */
  readonly bottomInset?: number;
};

export type FitFrame = {
  readonly target: Vector3;
  readonly distance: number;
};

/**
 * Frames `positions` for a camera looking in along `-direction` (unit vector
 * from target toward camera) with world-up `up`: the target is the center of
 * the points' screen-plane extent, and the distance is the smallest one that
 * keeps every point (padded by `pad`) inside both FOV axes. A bounding sphere
 * would overstate graphs that are deep but narrow from this bearing.
 */
export function fitFrame(
  positions: readonly Vector3[],
  direction: Vector3,
  up: Vector3,
  verticalFovDegrees: number,
  aspect: number,
  pad: number,
  bottomInset = 0,
): FitFrame {
  const right = new Vector3().crossVectors(up, direction);
  if (right.lengthSq() < MIN_TARGET_LENGTH) {
    right.set(1, 0, 0);
  }
  right.normalize();
  const screenUp = new Vector3().crossVectors(direction, right).normalize();
  const centroid = positions.reduce((sum, position) => sum.add(position), new Vector3()).divideScalar(positions.length);
  const coords = positions.map((position) => {
    const offset = position.clone().sub(centroid);
    return { x: offset.dot(right), y: offset.dot(screenUp), depth: offset.dot(direction) };
  });
  const midX = (Math.min(...coords.map((c) => c.x)) + Math.max(...coords.map((c) => c.x))) / 2;
  const midY = (Math.min(...coords.map((c) => c.y)) + Math.max(...coords.map((c) => c.y))) / 2;
  const tanFull = Math.tan(verticalFovDegrees * Math.PI / 360);
  // A bottom inset (e.g. a caption) shrinks the usable vertical window and
  // moves its center up by tanFull * bottomInset.
  const tanVertical = tanFull * (1 - bottomInset);
  const tanHorizontal = tanFull * aspect;
  const distance = coords.reduce((best, c) => Math.max(
    best,
    c.depth + pad + (Math.abs(c.y - midY) + pad) / tanVertical,
    c.depth + pad + (Math.abs(c.x - midX) + pad) / tanHorizontal,
  ), MOTION_OFF);
  const target = centroid
    .add(right.multiplyScalar(midX))
    .add(screenUp.multiplyScalar(midY - tanFull * bottomInset * distance));
  return { target, distance };
}

const MOTION_ON = 1;
const MOTION_OFF = 0;
const ASPECT_FALLBACK = 1;
const MIN_TARGET_LENGTH = 0.0001;
const DRIFT_WAKE_SLACK_MS = 20;
const FIT_DISTANCE_HEADROOM = 1.15;
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
  /** performance.now() of the last interaction or scripted camera move. */
  lastActivityAt: number;
  /** Wakes the render loop when the idle delay ends, instead of polling per frame. */
  driftWake: ReturnType<typeof setTimeout> | null;
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
  controls.minDistance = RENDER_TOKENS.camera.minDistance;
  controls.maxDistance = RENDER_TOKENS.camera.maxDistance;
  controls.maxTargetRadius = RENDER_TOKENS.camera.maxTargetRadius;
  controls.target.copy(TARGET_ORIGIN);
  const rigState: RigState = {
    flight: null,
    driftEnabled: false,
    reducedMotion: false,
    interacting: false,
    lastActivityAt: performance.now(),
    driftWake: null,
  };
  const onStart = (): void => {
    // Direct manipulation wins over a scripted flight instead of fighting it.
    rigState.flight = null;
    rigState.interacting = true;
    rigState.lastActivityAt = performance.now();
  };
  const onEnd = (): void => {
    rigState.interacting = false;
    rigState.lastActivityAt = performance.now();
  };
  const onChange = (): void => onViewChange?.();
  controls.addEventListener("start", onStart);
  controls.addEventListener("end", onEnd);
  controls.addEventListener("change", onChange);
  return {
    camera,
    controls,
    flyTo: (position, reducedMotion) => {
      rigState.lastActivityAt = performance.now();
      rigState.flight = beginFlight(camera, controls, position, reducedMotion);
    },
    zoomToFit: (positions, opts) => {
      if (positions.length === 0) {
        return;
      }
      rigState.lastActivityAt = performance.now();
      rigState.flight = beginZoomToFit(camera, controls, positions, opts);
    },
    setDrift: (enabled, autoRotateSpeed = RENDER_TOKENS.drift.autoRotateSpeed) => {
      rigState.driftEnabled = enabled;
      controls.autoRotateSpeed = autoRotateSpeed;
    },
    setReducedMotion: (reduced) => {
      applyReducedMotion(camera, controls, rigState, reduced);
    },
    update: (deltaSeconds) => updateRig(camera, controls, rigState, deltaSeconds, onViewChange),
    dispose: () => {
      if (rigState.driftWake !== null) {
        clearTimeout(rigState.driftWake);
      }
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
  rigState: RigState,
  reduced: boolean,
): void {
  rigState.reducedMotion = reduced;
  controls.enableDamping = motionSettings(reduced).dampingEnabled;
  if (reduced && rigState.flight !== null) {
    // An in-flight animation snaps to its destination; the end state is kept.
    camera.position.copy(rigState.flight.toPosition);
    controls.target.copy(rigState.flight.toTarget);
    controls.update();
    rigState.flight = null;
  }
}

function beginFlight(
  camera: PerspectiveCamera,
  controls: OrbitControls,
  target: Vector3,
  reducedMotion: boolean,
): Flight | null {
  const direction = target.length() > MIN_TARGET_LENGTH ? target.clone().normalize() : new Vector3(0, 0, 1);
  const toPosition = target.clone().add(direction.multiplyScalar(RENDER_TOKENS.camera.targetDistance));
  return beginFlightTo(
    camera,
    controls,
    toPosition,
    target,
    flightDurationMs(camera.position.distanceTo(toPosition)),
    reducedMotion,
    RENDER_TOKENS.camera.maxDurationMs,
  );
}

function beginFlightTo(
  camera: PerspectiveCamera,
  controls: OrbitControls,
  toPosition: Vector3,
  toTarget: Vector3,
  durationMs: number,
  reducedMotion: boolean,
  maxDurationMs: number,
): Flight | null {
  const settings = motionSettings(reducedMotion);
  controls.enableDamping = settings.dampingEnabled;
  if (settings.durationMs === MOTION_OFF) {
    camera.position.copy(toPosition);
    controls.target.copy(toTarget);
    controls.update();
    return null;
  }
  return {
    fromPosition: camera.position.clone(),
    toPosition,
    fromTarget: controls.target.clone(),
    toTarget: toTarget.clone(),
    durationMs: Math.min(durationMs, maxDurationMs),
    elapsedMs: MOTION_OFF,
  };
}

function beginZoomToFit(
  camera: PerspectiveCamera,
  controls: OrbitControls,
  positions: readonly Vector3[],
  opts: ZoomToFitOptions,
): Flight | null {
  const centroid = positions.reduce((sum, position) => sum.add(position), new Vector3()).divideScalar(positions.length);
  const direction = fitDirection(camera, controls, centroid, opts.bearing ?? "centroid");
  const frame = fitFrame(
    positions,
    direction,
    camera.up,
    camera.fov,
    camera.aspect,
    opts.paddingWorldUnits + RENDER_TOKENS.node.maxRadius,
    opts.bottomInset ?? 0,
  );
  // A fit that needs more room than the default bound (a tall narrow viewport)
  // widens it; OrbitControls would otherwise clamp the flight short.
  controls.maxDistance = Math.max(RENDER_TOKENS.camera.maxDistance, frame.distance * FIT_DISTANCE_HEADROOM);
  const toPosition = frame.target.clone().add(direction.multiplyScalar(frame.distance));
  return beginFlightTo(
    camera,
    controls,
    toPosition,
    frame.target,
    RENDER_TOKENS.camera.beatDurationMs,
    opts.reducedMotion,
    RENDER_TOKENS.camera.beatDurationMs,
  );
}

function fitDirection(
  camera: PerspectiveCamera,
  controls: OrbitControls,
  centroid: Vector3,
  bearing: "centroid" | "current",
): Vector3 {
  const source = bearing === "current" ? camera.position.clone().sub(controls.target) : centroid.clone();
  return source.length() > MIN_TARGET_LENGTH ? source.normalize() : new Vector3(0, 0, 1);
}

export function zoomToFit(
  rig: CameraRig,
  positions: readonly Vector3[],
  opts: ZoomToFitOptions,
): void {
  rig.zoomToFit(positions, opts);
}

function advanceFlight(
  camera: PerspectiveCamera,
  controls: OrbitControls,
  rigState: RigState,
  deltaMs: number,
): void {
  const flight = rigState.flight;
  if (flight === null) {
    return;
  }
  flight.elapsedMs = Math.min(flight.durationMs, flight.elapsedMs + deltaMs);
  const t = flight.elapsedMs / flight.durationMs;
  camera.position.lerpVectors(flight.fromPosition, flight.toPosition, easeOutQuad(t));
  const targetT = Math.min(MOTION_ON, t * RENDER_TOKENS.camera.aimLockFraction);
  controls.target.lerpVectors(flight.fromTarget, flight.toTarget, easeOutQuad(targetT));
  if (flight.elapsedMs >= flight.durationMs) {
    rigState.flight = null;
  }
}

function updateRig(
  camera: PerspectiveCamera,
  controls: OrbitControls,
  rigState: RigState,
  deltaSeconds: number,
  onWake?: () => void,
): boolean {
  const deltaMs = deltaSeconds * RENDER_TOKENS.time.secondsToMs;
  const now = performance.now();
  if (rigState.flight !== null) {
    advanceFlight(camera, controls, rigState, deltaMs);
    controls.update();
    rigState.lastActivityAt = now;
    return true;
  }
  const idleMs = now - rigState.lastActivityAt;
  controls.autoRotate = driftActive(rigState.driftEnabled, rigState.interacting, idleMs, rigState.reducedMotion);
  // OrbitControls.update reports whether the camera moved, so settled damping
  // stops the loop.
  const moved = controls.update(deltaSeconds);
  const awaitingDrift = rigState.driftEnabled && !rigState.interacting && !rigState.reducedMotion && !controls.autoRotate;
  if (awaitingDrift && rigState.driftWake === null && onWake !== undefined) {
    // One timer for the rest of the idle delay; the woken frame re-evaluates
    // (and re-arms if activity happened meanwhile).
    rigState.driftWake = setTimeout(() => {
      rigState.driftWake = null;
      onWake();
    }, Math.max(MOTION_OFF, RENDER_TOKENS.drift.idleDelayMs - idleMs) + DRIFT_WAKE_SLACK_MS);
  }
  return moved || controls.autoRotate;
}
