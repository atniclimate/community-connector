import { Vector3 } from "three";
import type { ProjectionEntityDto } from "../state/state";
import { RENDER_TOKENS } from "./config";

export type EntityPosition = {
  readonly id: string;
  readonly position: Vector3;
};

export type LayoutResult = {
  readonly positions: ReadonlyMap<string, Vector3>;
};

const FNV_OFFSET_BASIS = 2166136261;
const FNV_PRIME = 16777619;
const UINT32_MAX = 0xffffffff;
const UNIT = 1;
const DIAMETER = 2;
const UNKNOWN_KIND = "unknown";
const HASH_Y_SUFFIX = ":y";
const HASH_THETA_SUFFIX = ":theta";
const SNAPSHOT_DECIMALS = 6;

function hashString(value: string): number {
  let hash = FNV_OFFSET_BASIS;
  for (const char of value) {
    hash ^= char.charCodeAt(0);
    hash = Math.imul(hash, FNV_PRIME);
  }
  return hash >>> 0;
}

function unitFromHash(hash: number): number {
  return hash / UINT32_MAX;
}

function clusterCenter(kind: string): Vector3 {
  const hash = hashString(kind);
  const index = hash % RENDER_TOKENS.layout.radius;
  const y = UNIT - (index / RENDER_TOKENS.layout.radius) * DIAMETER;
  const radius = Math.sqrt(Math.max(0, UNIT - y * y));
  const theta = index * RENDER_TOKENS.layout.goldenAngle;
  return new Vector3(Math.cos(theta) * radius, y, Math.sin(theta) * radius)
    .multiplyScalar(RENDER_TOKENS.layout.clusterRadius);
}

function spikeOffset(entity: ProjectionEntityDto): Vector3 {
  const hash = hashString(entity.id);
  const jitter =
    RENDER_TOKENS.layout.minJitter +
    unitFromHash(hash) * (RENDER_TOKENS.layout.maxJitter - RENDER_TOKENS.layout.minJitter);
  const y = UNIT - unitFromHash(hashString(`${entity.id}${HASH_Y_SUFFIX}`)) * DIAMETER;
  const radius = Math.sqrt(Math.max(0, UNIT - y * y));
  const theta = hashString(`${entity.id}${HASH_THETA_SUFFIX}`) * RENDER_TOKENS.layout.goldenAngle;
  return new Vector3(Math.cos(theta) * radius, y, Math.sin(theta) * radius)
    .multiplyScalar(RENDER_TOKENS.layout.radius * jitter);
}

export function positionForEntity(entity: ProjectionEntityDto): Vector3 {
  return clusterCenter(entity.kind ?? UNKNOWN_KIND).add(spikeOffset(entity));
}

export function computeLayout(entities: readonly ProjectionEntityDto[]): LayoutResult {
  // Decision: v0 layout is deterministic client-side seeded spherical clusters.
  // Phase 4+ pipeline-precomputed layout should replace only this module.
  const ordered = [...entities].sort((left, right) => left.id.localeCompare(right.id));
  const positions = new Map<string, Vector3>();
  for (const entity of ordered) {
    positions.set(entity.id, positionForEntity(entity));
  }
  return { positions };
}

export function layoutSnapshot(layout: LayoutResult): Readonly<Record<string, readonly number[]>> {
  return Object.fromEntries(
    [...layout.positions.entries()].map(([id, position]) => [
      id,
      [position.x, position.y, position.z].map((value) => Number(value.toFixed(SNAPSHOT_DECIMALS))),
    ]),
  );
}
