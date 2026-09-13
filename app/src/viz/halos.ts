import {
  BackSide,
  Color,
  DynamicDrawUsage,
  Group,
  IcosahedronGeometry,
  InstancedMesh,
  Object3D,
  ShaderMaterial,
  UniformsLib,
  UniformsUtils,
  Vector3,
} from "three";
import type { KindMeta, ProjectionDto, ShapeName, ViewMode } from "../state/state";
import type { Theme } from "../theme/tokens";
import { RENDER_TOKENS } from "./config";
import type { LayoutResult } from "./layout";
import { degreeByEntityId, kindColor, projectedEdges } from "./projection";
import { degreeToScale } from "./nodes";
import type { QualityTier } from "./quality";

export type HaloLayer = {
  /** Scene root: the resting field plus the selected shell. */
  readonly group: Group;
  /** Per-kind resting halos; draw counts follow the camera. */
  readonly field: Group;
  readonly selected: InstancedMesh;
  readonly restingAlpha: number;
  /** Re-picks the nearest visible resting halos for a camera position. */
  readonly refresh: (cameraPosition: Vector3) => void;
  readonly lastRefreshPosition: Vector3;
  /** Shows the selected shell around `entityId` (null hides it). */
  readonly setSelected: (entityId: string | null) => void;
  readonly dispose: () => void;
};

type HaloEntry = {
  readonly entityId: string;
  readonly kind: string;
  readonly position: Vector3;
  readonly scale: number;
};

export type HaloCandidate = {
  readonly entryIndex: number;
  readonly distanceSq: number;
};

const UNIT = 1;
const ZERO_VISIBLE = 0;
const SINGLE_INSTANCE = 1;
const MATRIX_COMPONENTS = 16;
const TORUS_HALO_RADIUS_FACTOR = 1.35;
const DUMMY = new Object3D();

function haloMaterial(color: Color, alpha: number, rim = false): ShaderMaterial {
  return new ShaderMaterial({
    side: BackSide,
    transparent: true,
    depthWrite: false,
    fog: true,
    uniforms: UniformsUtils.merge([
      UniformsLib.fog,
      {
        uColor: { value: color },
        uAlpha: { value: alpha },
        uFalloffC: { value: RENDER_TOKENS.halo.falloffC },
        uFalloffP: { value: RENDER_TOKENS.halo.falloffP },
        uRim: { value: rim ? UNIT : ZERO_VISIBLE },
      },
    ]),
    vertexShader: `
      #include <common>
      #include <fog_pars_vertex>
      varying vec3 vNormal;
      void main() {
        vNormal = normalize(normalMatrix * normal);
        vec4 mvPosition = modelViewMatrix * instanceMatrix * vec4(position, 1.0);
        gl_Position = projectionMatrix * mvPosition;
        #include <fog_vertex>
      }
    `,
    fragmentShader: `
      #include <common>
      #include <fog_pars_fragment>
      varying vec3 vNormal;
      uniform vec3 uColor;
      uniform float uAlpha;
      uniform float uFalloffC;
      uniform float uFalloffP;
      uniform float uRim;
      void main() {
        float facing = abs(dot(vNormal, vec3(0.0, 0.0, 1.0)));
        // Resting glow is brightest behind the node; the selected shell is an
        // outline ring, brightest at its silhouette.
        float glow = pow(uFalloffC + facing, uFalloffP);
        float ring = pow(1.0 - facing, ${RENDER_TOKENS.halo.selectedRimPower.toFixed(1)});
        float falloff = mix(glow, ring, uRim);
        gl_FragColor = vec4(uColor, clamp(uAlpha * falloff, 0.0, 1.0));
        #include <tonemapping_fragment>
        #include <colorspace_fragment>
        #include <fog_fragment>
      }
    `,
  });
}

function tierDistance(tier: QualityTier): number {
  if (tier === "A") {
    return RENDER_TOKENS.halo.tierADistance;
  }
  if (tier === "B") {
    return RENDER_TOKENS.halo.tierBDistance;
  }
  return ZERO_VISIBLE;
}

function visibleLimit(tier: QualityTier): number {
  return tier === "A" || tier === "B" ? RENDER_TOKENS.halo.maxVisible : ZERO_VISIBLE;
}

function shellScale(shape: ShapeName, nodeScale: number): number {
  const shapeFactor = shape === "torus" ? TORUS_HALO_RADIUS_FACTOR : UNIT;
  return nodeScale * RENDER_TOKENS.halo.scale * shapeFactor;
}

/** Distance eligibility first, then the tier cap (ADR-004: halos degrade first). */
export function selectHaloCandidates(
  positions: readonly Vector3[],
  cameraPosition: Vector3,
  tier: QualityTier,
): readonly HaloCandidate[] {
  const maxVisible = visibleLimit(tier);
  if (maxVisible === ZERO_VISIBLE) {
    return [];
  }
  const maxDistanceSq = tierDistance(tier) ** 2;
  const eligible: HaloCandidate[] = [];
  for (const [entryIndex, position] of positions.entries()) {
    const distanceSq = position.distanceToSquared(cameraPosition);
    if (distanceSq <= maxDistanceSq) {
      eligible.push({ entryIndex, distanceSq });
    }
  }
  return eligible.sort((left, right) => left.distanceSq - right.distanceSq).slice(ZERO_VISIBLE, maxVisible);
}

function writeShell(mesh: InstancedMesh, instanceId: number, entry: HaloEntry, scaleFactor = UNIT): void {
  DUMMY.position.copy(entry.position);
  DUMMY.scale.setScalar(entry.scale * scaleFactor);
  DUMMY.updateMatrix();
  mesh.setMatrixAt(instanceId, DUMMY.matrix);
}

export function buildHaloLayer(args: {
  readonly projection: ProjectionDto;
  readonly layout: LayoutResult;
  readonly kindMeta: Readonly<Record<string, KindMeta>>;
  readonly theme: Theme | null;
  readonly tier: QualityTier;
  readonly cameraPosition: Vector3;
  readonly viewMode: ViewMode;
}): HaloLayer {
  const group = new Group();
  const field = new Group();
  const restingAlpha = args.viewMode === "present"
    ? RENDER_TOKENS.halo.restingAlphaPresent
    : RENDER_TOKENS.halo.restingAlpha;
  const geometry = new IcosahedronGeometry(UNIT, RENDER_TOKENS.node.geometryDetail);
  const entities = args.projection.entities ?? [];
  const degrees = degreeByEntityId(entities, projectedEdges(args.projection));
  const entries: HaloEntry[] = [];
  const kindCounts = new Map<string, number>();
  for (const entity of entities) {
    const position = args.layout.positions.get(entity.id);
    if (position === undefined) {
      continue;
    }
    const kind = entity.kind ?? "";
    const shape = args.kindMeta[kind]?.shape ?? "sphere";
    const nodeScale = degreeToScale(degrees.get(entity.id) ?? RENDER_TOKENS.node.minDegree);
    entries.push({ entityId: entity.id, kind, position, scale: shellScale(shape, nodeScale) });
    kindCounts.set(kind, (kindCounts.get(kind) ?? ZERO_VISIBLE) + UNIT);
  }
  const kindMeshes = new Map<string, InstancedMesh>();
  for (const [kind, count] of kindCounts.entries()) {
    const mesh = new InstancedMesh(geometry, haloMaterial(new Color(kindColor(kind, args.theme)), restingAlpha), count);
    mesh.instanceMatrix.setUsage(DynamicDrawUsage);
    // Instances are reassigned on refresh; lazily cached bounds would go stale.
    mesh.frustumCulled = false;
    kindMeshes.set(kind, mesh);
    field.add(mesh);
  }
  // One instance only, so a smoother silhouette costs a few hundred triangles.
  const selectedGeometry = new IcosahedronGeometry(UNIT, RENDER_TOKENS.halo.selectedGeometryDetail);
  const selected = new InstancedMesh(
    selectedGeometry,
    haloMaterial(new Color(), RENDER_TOKENS.halo.selectedAlpha, true),
    SINGLE_INSTANCE,
  );
  selected.visible = false;
  selected.frustumCulled = false;
  group.add(field, selected);
  const lastRefreshPosition = args.cameraPosition.clone();
  const entryById = new Map(entries.map((entry) => [entry.entityId, entry]));
  const positions = entries.map((entry) => entry.position);

  const refresh = (cameraPosition: Vector3): void => {
    lastRefreshPosition.copy(cameraPosition);
    const used = new Map<string, number>();
    for (const candidate of selectHaloCandidates(positions, cameraPosition, args.tier)) {
      const entry = entries[candidate.entryIndex];
      const mesh = entry === undefined ? undefined : kindMeshes.get(entry.kind);
      if (entry === undefined || mesh === undefined) {
        continue;
      }
      const instanceId = used.get(entry.kind) ?? ZERO_VISIBLE;
      writeShell(mesh, instanceId, entry);
      used.set(entry.kind, instanceId + UNIT);
    }
    for (const [kind, mesh] of kindMeshes.entries()) {
      mesh.count = used.get(kind) ?? ZERO_VISIBLE;
      mesh.visible = mesh.count > ZERO_VISIBLE;
      if (mesh.visible) {
        // Upload only the rewritten prefix, not the full-capacity buffer.
        mesh.instanceMatrix.clearUpdateRanges();
        mesh.instanceMatrix.addUpdateRange(ZERO_VISIBLE, mesh.count * MATRIX_COMPONENTS);
        mesh.instanceMatrix.needsUpdate = true;
      }
    }
  };
  refresh(args.cameraPosition);

  return {
    group,
    field,
    selected,
    restingAlpha,
    refresh,
    lastRefreshPosition,
    setSelected: (entityId) => {
      const entry = entityId === null ? undefined : entryById.get(entityId);
      if (entry === undefined) {
        selected.visible = false;
        return;
      }
      // Clears the enlarged focused node, including cone/cube corners.
      writeShell(selected, ZERO_VISIBLE, entry, RENDER_TOKENS.halo.selectedShellFactor);
      selected.instanceMatrix.needsUpdate = true;
      const color = (selected.material as ShaderMaterial).uniforms.uColor?.value;
      if (color instanceof Color) {
        color.set(kindColor(entry.kind, args.theme));
      }
      selected.visible = true;
    },
    dispose: () => {
      geometry.dispose();
      selectedGeometry.dispose();
      for (const mesh of [...kindMeshes.values(), selected]) {
        (mesh.material as ShaderMaterial).dispose();
      }
    },
  };
}

/**
 * Dims the resting halo field as the focus blend rises (P1.2) and fades the
 * selected shell in on the same blend, so node, edge, and halo emphasis share
 * one timeline. Under reduced motion the blend snaps, so both snap.
 */
export function setHaloFocusDim(layer: HaloLayer, blend: number): void {
  const resting = layer.restingAlpha;
  const dimmed = resting * RENDER_TOKENS.focus.haloDimFactor;
  const alpha = resting + (dimmed - resting) * blend;
  for (const child of layer.field.children) {
    setAlpha(child as InstancedMesh, alpha);
  }
  setAlpha(layer.selected, RENDER_TOKENS.halo.selectedAlpha * blend);
}

function setAlpha(mesh: InstancedMesh, alpha: number): void {
  const uniform = (mesh.material as ShaderMaterial).uniforms.uAlpha;
  if (uniform === undefined) {
    throw new Error("Halo material is missing the uAlpha uniform");
  }
  uniform.value = alpha;
}
