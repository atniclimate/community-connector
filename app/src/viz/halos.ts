import {
  BackSide,
  Color,
  Group,
  IcosahedronGeometry,
  InstancedMesh,
  Object3D,
  ShaderMaterial,
  Vector3,
} from "three";
import type { KindMeta, ProjectionDto, ShapeName } from "../state/state";
import type { Theme } from "../theme/tokens";
import { RENDER_TOKENS } from "./config";
import type { LayoutResult } from "./layout";
import { degreeByEntityId, kindColor, projectedEdges } from "./projection";
import { degreeToScale } from "./nodes";
import type { QualityTier } from "./quality";

export type HaloLayer = {
  readonly group: Group;
  readonly dispose: () => void;
};

const UNIT = 1;
const BASE_VISIBLE_DISTANCE = Number.POSITIVE_INFINITY;
const ZERO_VISIBLE = 0;
const TORUS_HALO_RADIUS_FACTOR = 1.35;
const DUMMY = new Object3D();

type HaloCandidate = {
  readonly entityIndex: number;
  readonly distance: number;
};

function haloMaterial(color: Color): ShaderMaterial {
  return new ShaderMaterial({
    side: BackSide,
    transparent: true,
    depthWrite: false,
    uniforms: {
      uColor: { value: color },
      uAlpha: { value: RENDER_TOKENS.halo.restingAlpha },
      uFalloffC: { value: RENDER_TOKENS.halo.falloffC },
      uFalloffP: { value: RENDER_TOKENS.halo.falloffP },
    },
    vertexShader: `
      varying vec3 vNormal;
      void main() {
        vNormal = normalize(normalMatrix * normal);
        gl_Position = projectionMatrix * modelViewMatrix * instanceMatrix * vec4(position, 1.0);
      }
    `,
    fragmentShader: `
      varying vec3 vNormal;
      uniform vec3 uColor;
      uniform float uAlpha;
      uniform float uFalloffC;
      uniform float uFalloffP;
      void main() {
        float fresnel = pow(uFalloffC + abs(dot(vNormal, vec3(0.0, 0.0, 1.0))), uFalloffP);
        gl_FragColor = vec4(uColor, clamp(uAlpha * fresnel, 0.0, 1.0));
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

function haloCandidates(args: Parameters<typeof buildHaloLayer>[0]): readonly HaloCandidate[] {
  const maxDistance = tierDistance(args.tier);
  const maxVisible = visibleLimit(args.tier);
  const entities = args.projection.entities ?? [];
  const candidates = entities.flatMap((entity, entityIndex) => {
    const position = args.layout.positions.get(entity.id);
    if (position === undefined) {
      return [];
    }
    return [{ entityIndex, distance: position.distanceTo(args.cameraPosition) }];
  }).sort((left, right) => left.distance - right.distance);
  return candidates
    .filter((candidate, index) => candidate.distance <= maxDistance || index < maxVisible)
    .slice(ZERO_VISIBLE, maxVisible);
}

export function buildHaloLayer(args: {
  readonly projection: ProjectionDto;
  readonly layout: LayoutResult;
  readonly kindMeta: Readonly<Record<string, KindMeta>>;
  readonly theme: Theme | null;
  readonly tier: QualityTier;
  readonly cameraPosition: Vector3;
}): HaloLayer {
  const group = new Group();
  const geometry = new IcosahedronGeometry(UNIT, RENDER_TOKENS.node.geometryDetail);
  const byKind = new Map<string, number[]>();
  const entities = args.projection.entities ?? [];
  for (const candidate of haloCandidates(args)) {
    const entity = entities[candidate.entityIndex];
    if (entity !== undefined) {
      byKind.set(entity.kind ?? "", [...(byKind.get(entity.kind ?? "") ?? []), candidate.entityIndex]);
    }
  }
  for (const [kind, indexes] of byKind.entries()) {
    addKindHalos(group, geometry, args, kind, indexes);
  }
  return {
    group,
    dispose: () => {
      geometry.dispose();
      for (const child of group.children) {
        const mesh = child as InstancedMesh;
        (mesh.material as ShaderMaterial).dispose();
      }
    },
  };
}

function addKindHalos(
  group: Group,
  geometry: IcosahedronGeometry,
  args: Parameters<typeof buildHaloLayer>[0],
  kind: string,
  indexes: readonly number[],
): void {
  const mesh = new InstancedMesh(geometry, haloMaterial(new Color(kindColor(kind, args.theme))), indexes.length);
  const entities = args.projection.entities ?? [];
  const shape = args.kindMeta[kind]?.shape ?? "sphere";
  const degrees = degreeByEntityId(entities, projectedEdges(args.projection));
  for (const [instanceId, entityIndex] of indexes.entries()) {
    const entity = entities[entityIndex];
    if (entity === undefined) {
      continue;
    }
    const position = args.layout.positions.get(entity.id);
    if (position === undefined) {
      continue;
    }
    DUMMY.position.copy(position);
    DUMMY.scale.setScalar(shellScale(shape, degreeToScale(degrees.get(entity.id) ?? RENDER_TOKENS.node.minDegree)));
    DUMMY.updateMatrix();
    mesh.setMatrixAt(instanceId, DUMMY.matrix);
  }
  mesh.instanceMatrix.needsUpdate = true;
  mesh.matrixWorldNeedsUpdate = true;
  group.add(mesh);
}
