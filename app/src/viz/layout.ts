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
    .multiplyScalar(RENDER_TOKENS.layout.radius);
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
    .multiplyScalar(RENDER_TOKENS.layout.clusterRadius * jitter);
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

const HASH_RING_SUFFIX = ":ring";
const HALF = 0.5;
/** Depth jitter as a share of ring spacing, so rings keep some 3D body. */
const EGO_DEPTH_SHARE = 0.2;

function byId(left: ProjectionEntityDto, right: ProjectionEntityDto): number {
  return left.id.localeCompare(right.id);
}

/**
 * Rings around one entity: the center at the origin, ring k for hop k of the
 * neighborhood layers, and every entity no layer reached on one outer ring.
 * Only ids in `entities` are placed, so a hidden entity leaves no gap (A-B4).
 */
export function computeEgoLayout(
  entities: readonly ProjectionEntityDto[],
  centerId: string,
  layers: readonly (readonly string[])[],
): LayoutResult {
  const ringOf = new Map<string, number>();
  layers.forEach((layer, index) => {
    for (const id of layer) {
      if (!ringOf.has(id)) {
        ringOf.set(id, index + UNIT);
      }
    }
  });
  const outer = layers.length + UNIT;
  const rings = new Map<number, string[]>();
  const positions = new Map<string, Vector3>();
  for (const entity of [...entities].sort(byId)) {
    if (entity.id === centerId) {
      positions.set(entity.id, new Vector3());
      continue;
    }
    const ring = ringOf.get(entity.id) ?? outer;
    const members = rings.get(ring) ?? [];
    members.push(entity.id);
    rings.set(ring, members);
  }
  const spacing = RENDER_TOKENS.layout.radius / outer;
  for (const [ring, ids] of rings) {
    const offset = unitFromHash(hashString(`${centerId}:${ring}${HASH_RING_SUFFIX}`)) * Math.PI * DIAMETER;
    ids.forEach((id, index) => {
      const theta = offset + index * RENDER_TOKENS.layout.goldenAngle;
      const depth = (unitFromHash(hashString(`${id}${HASH_Y_SUFFIX}`)) - HALF) * spacing * EGO_DEPTH_SHARE;
      positions.set(id, new Vector3(Math.cos(theta), Math.sin(theta), 0).multiplyScalar(ring * spacing).setZ(depth));
    });
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
