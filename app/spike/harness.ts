import * as THREE from "three";
import type { SpikeData } from "./data";

export type CandidateName = "stock" | "forcegraph" | "instanced";

export type SpikeResult = {
  candidate: CandidateName;
  frames: number;
  avgFps: number;
  p95FrameMs: number;
  worstFrameMs: number;
  drawCalls: number | null;
  userAgent: string;
  timestamp: string;
};

export type CandidateHandle = {
  renderer?: THREE.WebGLRenderer;
  setCamera: (position: THREE.Vector3, lookAt: THREE.Vector3) => void;
  render?: () => void;
  dispose: () => void;
};

export type CandidateFactory = (container: HTMLElement, data: SpikeData) => CandidateHandle;

const WARMUP_MS = 3_000;
const MEASUREMENT_MS = 20_000;
const STORAGE_KEY = "community-connector-rendering-spike";
const ORDER: CandidateName[] = ["stock", "forcegraph", "instanced"];
const LOOK_AT = new THREE.Vector3(0, 0, 0);

export function getCandidateFromUrl(): CandidateName | null {
  const candidate = new URLSearchParams(window.location.search).get("candidate");
  if (candidate === "stock" || candidate === "forcegraph" || candidate === "instanced") {
    return candidate;
  }
  return null;
}

export function isAutoRun(): boolean {
  return new URLSearchParams(window.location.search).get("auto") === "1";
}

export async function runMeasurement(candidate: CandidateName, handle: CandidateHandle): Promise<SpikeResult> {
  const frameTimes: number[] = [];
  let frames = 0;
  let previous = performance.now();
  const started = previous;
  const measuredAt = started + WARMUP_MS;
  const finishedAt = measuredAt + MEASUREMENT_MS;

  return await new Promise((resolve) => {
    const tick = (now: number): void => {
      const elapsed = now - started;
      moveCamera(handle, elapsed);
      handle.render?.();

      if (now >= measuredAt) {
        frameTimes.push(now - previous);
        frames += 1;
      }

      previous = now;

      if (now < finishedAt) {
        window.requestAnimationFrame(tick);
        return;
      }

      const avgFps = frames / (MEASUREMENT_MS / 1_000);
      const sorted = [...frameTimes].sort((a, b) => a - b);
      const p95Index = Math.min(sorted.length - 1, Math.floor(sorted.length * 0.95));
      const p95FrameMs = sorted[p95Index] ?? 0;
      const worstFrameMs = sorted.at(-1) ?? 0;

      resolve({
        candidate,
        frames,
        avgFps: round(avgFps),
        p95FrameMs: round(p95FrameMs),
        worstFrameMs: round(worstFrameMs),
        drawCalls: handle.renderer?.info.render.calls ?? null,
        userAgent: navigator.userAgent,
        timestamp: new Date().toISOString(),
      });
    };

    window.requestAnimationFrame(tick);
  });
}

export function logResult(result: SpikeResult): void {
  console.log("SPIKE_RESULT", JSON.stringify(result));
}

export function continueAutoRun(result: SpikeResult): boolean {
  const existing = readStoredResults();
  const nextResults = [...existing.filter((item) => item.candidate !== result.candidate), result];
  localStorage.setItem(STORAGE_KEY, JSON.stringify(nextResults));

  const nextCandidate = ORDER.find((name) => nextResults.every((item) => item.candidate !== name));
  if (nextCandidate === undefined) {
    console.log("SPIKE_ALL", JSON.stringify(nextResults));
    return false;
  }

  window.location.href = `${window.location.pathname}?auto=1&candidate=${nextCandidate}`;
  return true;
}

export function startAutoRun(): void {
  localStorage.removeItem(STORAGE_KEY);
  window.location.href = `${window.location.pathname}?auto=1&candidate=${ORDER[0]}`;
}

export function readStoredResults(): SpikeResult[] {
  const raw = localStorage.getItem(STORAGE_KEY);
  if (raw === null) {
    return [];
  }

  const parsed: unknown = JSON.parse(raw);
  if (!Array.isArray(parsed)) {
    throw new Error("Stored spike results are not an array");
  }
  return parsed.map(parseResult);
}

function moveCamera(handle: CandidateHandle, elapsedMs: number): void {
  const t = Math.min(elapsedMs - WARMUP_MS, MEASUREMENT_MS);
  const clamped = Math.max(0, t);
  const angle = clamped * 0.00022;
  const radius = cameraRadius(clamped);
  const y = radius * 0.26;
  const x = Math.cos(angle) * radius;
  const z = Math.sin(angle) * radius;

  handle.setCamera(new THREE.Vector3(x, y, z), LOOK_AT);
}

function cameraRadius(elapsedMs: number): number {
  if (elapsedMs <= 8_000) {
    return 1_200;
  }
  if (elapsedMs <= 14_000) {
    const progress = (elapsedMs - 8_000) / 6_000;
    return 1_200 + (500 - 1_200) * progress;
  }
  return 500;
}

function parseResult(value: unknown): SpikeResult {
  if (!isResult(value)) {
    throw new Error("Stored spike result has an invalid shape");
  }
  return value;
}

function isResult(value: unknown): value is SpikeResult {
  if (typeof value !== "object" || value === null) {
    return false;
  }
  const candidate = (value as Partial<SpikeResult>).candidate;
  return candidate === "stock" || candidate === "forcegraph" || candidate === "instanced";
}

function round(value: number): number {
  return Math.round(value * 100) / 100;
}
