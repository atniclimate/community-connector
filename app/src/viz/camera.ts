import { MathUtils, PerspectiveCamera, Vector3 } from "three";
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

function easeOutQuad(t: number): number {
  return t * (2 - t);
}

export function createCameraRig(canvas: HTMLCanvasElement): CameraRig {
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
  controls.target.copy(TARGET_ORIGIN);
  let flight: Flight | null = null;
  return {
    camera,
    controls,
    flyTo: (position, reducedMotion) => {
      flight = beginFlight(camera, controls, position, reducedMotion);
    },
    update: (deltaSeconds) => updateRig(camera, controls, deltaSeconds, flight, (next) => { flight = next; }),
    dispose: () => controls.dispose(),
  };
}

type Flight = {
  readonly fromPosition: Vector3;
  readonly toPosition: Vector3;
  readonly fromTarget: Vector3;
  readonly toTarget: Vector3;
  readonly durationMs: number;
  elapsedMs: number;
};

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
    durationMs: Math.min(settings.durationMs, RENDER_TOKENS.camera.maxDurationMs),
    elapsedMs: MOTION_OFF,
  };
}

function updateRig(
  camera: PerspectiveCamera,
  controls: OrbitControls,
  deltaSeconds: number,
  flight: Flight | null,
  setFlight: (flight: Flight | null) => void,
): boolean {
  if (flight !== null) {
    const deltaMs = deltaSeconds * RENDER_TOKENS.time.secondsToMs;
    flight.elapsedMs = Math.min(flight.durationMs, flight.elapsedMs + deltaMs);
    const t = flight.elapsedMs / flight.durationMs;
    camera.position.lerpVectors(flight.fromPosition, flight.toPosition, easeOutQuad(t));
    const targetT = Math.min(MOTION_ON, t * RENDER_TOKENS.camera.aimLockFraction);
    controls.target.lerpVectors(flight.fromTarget, flight.toTarget, easeOutQuad(targetT));
    if (flight.elapsedMs >= flight.durationMs) {
      setFlight(null);
    }
    controls.update();
    return true;
  }
  return controls.enableDamping && MathUtils.damp(MOTION_OFF, MOTION_ON, RENDER_TOKENS.camera.dampLambda, deltaSeconds) > MOTION_OFF;
}
