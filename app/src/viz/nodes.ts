import {
  BoxGeometry,
  ConeGeometry,
  DynamicDrawUsage,
  Color,
  Group,
  IcosahedronGeometry,
  InstancedMesh,
  Matrix4,
  MeshLambertMaterial,
  Object3D,
  OctahedronGeometry,
  SphereGeometry,
  TetrahedronGeometry,
  TorusGeometry,
  type BufferGeometry,
} from "three";
import type { KindMeta, ProjectionDto, ProjectionEntityDto, ShapeName } from "../state/state";
import type { Theme } from "../theme/tokens";
import { RENDER_TOKENS } from "./config";
import type { LayoutResult } from "./layout";
import { degreeByEntityId, kindColor, projectedEdges } from "./projection";

export type NodeMeshRecord = {
  readonly shape: ShapeName;
  readonly mesh: InstancedMesh;
  readonly entityIds: readonly string[];
};

export type NodeLayer = {
  readonly group: Group;
  readonly records: readonly NodeMeshRecord[];
  readonly entityToMesh: ReadonlyMap<string, { readonly mesh: InstancedMesh; readonly instanceId: number }>;
  readonly meshToEntityIds: ReadonlyMap<InstancedMesh, readonly string[]>;
  readonly dispose: () => void;
};

const UNIT = 1;
const RADIAL_SEGMENTS = 6;
const TORUS_TUBE_RADIUS = 0.35;
const TORUS_RADIAL_SEGMENTS = 6;
const TORUS_TUBULAR_SEGMENTS = 12;
const CONE_RADIUS = 1;
const CONE_HEIGHT = 2;
const MATRIX = new Matrix4();
const DUMMY = new Object3D();

function shapeGeometry(shape: ShapeName): BufferGeometry {
  switch (shape) {
    case "sphere":
      return new IcosahedronGeometry(UNIT, RENDER_TOKENS.node.geometryDetail);
    case "cube":
      return new BoxGeometry(UNIT, UNIT, UNIT);
    case "octahedron":
      return new OctahedronGeometry(UNIT);
    case "tetrahedron":
      return new TetrahedronGeometry(UNIT);
    case "torus":
      return new TorusGeometry(UNIT, TORUS_TUBE_RADIUS, TORUS_RADIAL_SEGMENTS, TORUS_TUBULAR_SEGMENTS);
    case "cone":
      return new ConeGeometry(CONE_RADIUS, CONE_HEIGHT, RADIAL_SEGMENTS);
    default:
      return new SphereGeometry(UNIT, RADIAL_SEGMENTS, RADIAL_SEGMENTS);
  }
}

export function degreeToScale(degree: number): number {
  const clamped = Math.min(
    RENDER_TOKENS.node.referenceDegree,
    Math.max(RENDER_TOKENS.node.minDegree, degree),
  );
  const t = clamped / RENDER_TOKENS.node.referenceDegree;
  return RENDER_TOKENS.node.minRadius + t * (RENDER_TOKENS.node.maxRadius - RENDER_TOKENS.node.minRadius);
}

function material(): MeshLambertMaterial {
  return new MeshLambertMaterial({
    vertexColors: true,
    emissive: new Color(RENDER_TOKENS.node.minDegree, RENDER_TOKENS.node.minDegree, RENDER_TOKENS.node.minDegree),
  });
}

function meshForShape(shape: ShapeName, count: number): InstancedMesh {
  const mesh = new InstancedMesh(shapeGeometry(shape), material(), count);
  mesh.instanceMatrix.setUsage(DynamicDrawUsage);
  mesh.instanceColor?.setUsage(DynamicDrawUsage);
  return mesh;
}

function groupByShape(
  entities: readonly ProjectionEntityDto[],
  meta: Readonly<Record<string, KindMeta>>,
): ReadonlyMap<ShapeName, readonly ProjectionEntityDto[]> {
  const groups = new Map<ShapeName, ProjectionEntityDto[]>();
  for (const entity of entities) {
    const shape = meta[entity.kind ?? ""]?.shape ?? "sphere";
    groups.set(shape, [...(groups.get(shape) ?? []), entity]);
  }
  return groups;
}

function setInstance(
  mesh: InstancedMesh,
  instanceId: number,
  entity: ProjectionEntityDto,
  layout: LayoutResult,
  scale: number,
  color: Color,
): void {
  const position = layout.positions.get(entity.id);
  if (position === undefined) {
    return;
  }
  DUMMY.position.copy(position);
  DUMMY.scale.setScalar(scale);
  DUMMY.updateMatrix();
  mesh.setMatrixAt(instanceId, DUMMY.matrix);
  mesh.setColorAt(instanceId, color);
}

function buildRecord(
  shape: ShapeName,
  entities: readonly ProjectionEntityDto[],
  args: BuildNodeLayerArgs,
): NodeMeshRecord {
  const mesh = meshForShape(shape, entities.length);
  const ids: string[] = [];
  for (const [instanceId, entity] of entities.entries()) {
    const color = new Color(kindColor(entity.kind ?? "", args.theme));
    const scale = degreeToScale(args.degrees.get(entity.id) ?? RENDER_TOKENS.node.minDegree);
    setInstance(mesh, instanceId, entity, args.layout, scale, color);
    ids.push(entity.id);
  }
  mesh.instanceMatrix.needsUpdate = true;
  if (mesh.instanceColor !== null) {
    mesh.instanceColor.needsUpdate = true;
  }
  return { shape, mesh, entityIds: ids };
}

export type BuildNodeLayerArgs = {
  readonly projection: ProjectionDto;
  readonly layout: LayoutResult;
  readonly kindMeta: Readonly<Record<string, KindMeta>>;
  readonly theme: Theme | null;
  readonly degrees: ReadonlyMap<string, number>;
};

export function buildNodeLayer(args: BuildNodeLayerArgs): NodeLayer {
  const group = new Group();
  const entities = args.projection.entities ?? [];
  const records = [...groupByShape(entities, args.kindMeta).entries()].map(([shape, shapeEntities]) =>
    buildRecord(shape, shapeEntities, args),
  );
  const entityToMesh = new Map<string, { mesh: InstancedMesh; instanceId: number }>();
  const meshToEntityIds = new Map<InstancedMesh, readonly string[]>();
  for (const record of records) {
    group.add(record.mesh);
    meshToEntityIds.set(record.mesh, record.entityIds);
    record.entityIds.forEach((id, instanceId) => entityToMesh.set(id, { mesh: record.mesh, instanceId }));
  }
  return { group, records, entityToMesh, meshToEntityIds, dispose: () => disposeRecords(records) };
}

export function degreesForProjection(projection: ProjectionDto): ReadonlyMap<string, number> {
  return degreeByEntityId(projection.entities ?? [], projectedEdges(projection));
}

export function recolorNodeLayer(layer: NodeLayer, projection: ProjectionDto, theme: Theme | null): void {
  const entities = new Map((projection.entities ?? []).map((entity) => [entity.id, entity]));
  for (const record of layer.records) {
    for (const [instanceId, entityId] of record.entityIds.entries()) {
      const entity = entities.get(entityId);
      if (entity !== undefined) {
        record.mesh.setColorAt(instanceId, new Color(kindColor(entity.kind ?? "", theme)));
      }
    }
    if (record.mesh.instanceColor !== null) {
      record.mesh.instanceColor.needsUpdate = true;
    }
  }
}

export function writeNodeState(layer: NodeLayer, entityId: string, state: "base" | "selected" | "dimmed"): void {
  const target = layer.entityToMesh.get(entityId);
  if (target === undefined) {
    return;
  }
  target.mesh.getMatrixAt(target.instanceId, MATRIX);
  MATRIX.decompose(DUMMY.position, DUMMY.quaternion, DUMMY.scale);
  const factor = stateFactor(state);
  DUMMY.scale.setScalar(Math.max(RENDER_TOKENS.node.minRadius, DUMMY.scale.x * factor));
  DUMMY.updateMatrix();
  target.mesh.setMatrixAt(target.instanceId, DUMMY.matrix);
  target.mesh.instanceMatrix.needsUpdate = true;
}

function stateFactor(state: "base" | "selected" | "dimmed"): number {
  if (state === "selected") {
    return RENDER_TOKENS.node.selectedScale;
  }
  if (state === "dimmed") {
    return RENDER_TOKENS.node.dimmedScale;
  }
  return UNIT;
}

function disposeRecords(records: readonly NodeMeshRecord[]): void {
  for (const record of records) {
    record.mesh.geometry.dispose();
    const materialValue = record.mesh.material;
    if (Array.isArray(materialValue)) {
      materialValue.forEach((item) => item.dispose());
    } else {
      materialValue.dispose();
    }
  }
}
