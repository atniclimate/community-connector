import {
  BoxGeometry,
  ConeGeometry,
  DynamicDrawUsage,
  Color,
  Group,
  IcosahedronGeometry,
  InstancedMesh,
  Matrix3,
  Matrix4,
  Object3D,
  OctahedronGeometry,
  ShaderMaterial,
  Sphere,
  SphereGeometry,
  TetrahedronGeometry,
  TorusGeometry,
  UniformsLib,
  UniformsUtils,
  Vector3,
  type BufferGeometry,
  type Intersection,
  type Raycaster,
} from "three";
import type { KindMeta, ProjectionDto, ProjectionEntityDto, ShapeName } from "../state/state";
import type { Theme } from "../theme/tokens";
import { RENDER_COLORS, RENDER_TOKENS } from "./config";
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
const TORUS_RADIAL_SEGMENTS = 10;
const TORUS_TUBULAR_SEGMENTS = 32;
/** Outer radius of the unit torus (ring radius + tube radius). */
export const TORUS_OUTER_RADIUS = UNIT + TORUS_TUBE_RADIUS;
const FACE_TILT_RADIANS = -0.45;
const CONE_RADIUS = 1;
const CONE_HEIGHT = 2;
const FULL_COLOR_SHARE = 1;
const NODE_DIFFUSE_SHARE = FULL_COLOR_SHARE - RENDER_TOKENS.node.emissiveShare;
const KEY_DIRECTION_X = -0.35;
const KEY_DIRECTION_Y = 0.8;
const KEY_DIRECTION_Z = 0.45;
const FILL_DIRECTION_X = 0.45;
const FILL_DIRECTION_Y = -0.55;
const FILL_DIRECTION_Z = 0.65;
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

function material(faceCamera: boolean): ShaderMaterial {
  return new ShaderMaterial({
    fog: true,
    // A ring seen edge-on reads as a pill, and orbiting guarantees some rings
    // are edge-on; FACE_CAMERA keeps the ring turned toward the viewer.
    defines: faceCamera ? { FACE_CAMERA: "" } : {},
    uniforms: UniformsUtils.merge([
      UniformsLib.fog,
      {
        uDiffuseShare: { value: NODE_DIFFUSE_SHARE },
        uNodeEmissiveShare: { value: RENDER_TOKENS.node.emissiveShare },
        uKeyColor: { value: new Color(RENDER_COLORS.keyLight) },
        uFillColor: { value: new Color(RENDER_COLORS.fillLight) },
        uKeyDirection: { value: new Vector3(KEY_DIRECTION_X, KEY_DIRECTION_Y, KEY_DIRECTION_Z) },
        uFillDirection: { value: new Vector3(FILL_DIRECTION_X, FILL_DIRECTION_Y, FILL_DIRECTION_Z) },
        uKeyIntensity: { value: RENDER_TOKENS.scene.keyIntensity },
        uFillIntensity: { value: RENDER_TOKENS.scene.fillIntensity },
        uAmbientIntensity: { value: RENDER_TOKENS.scene.ambientIntensity },
        uFaceTilt: { value: new Matrix3().set(
          1, 0, 0,
          0, Math.cos(FACE_TILT_RADIANS), -Math.sin(FACE_TILT_RADIANS),
          0, Math.sin(FACE_TILT_RADIANS), Math.cos(FACE_TILT_RADIANS),
        ) },
      },
    ]),
    vertexShader: `
      #include <common>
      #include <fog_pars_vertex>
      varying vec3 vNodeColor;
      varying vec3 vNodeNormal;
      uniform mat3 uFaceTilt;
      void main() {
        vNodeColor = instanceColor;
        #ifdef FACE_CAMERA
          // Geometry is placed in view space around the instance center, so
          // the ring always faces the camera, tilted for a readable 3D form.
          vec4 center = modelViewMatrix * instanceMatrix * vec4(0.0, 0.0, 0.0, 1.0);
          float nodeScale = length(instanceMatrix[0].xyz);
          vNodeNormal = normalize(uFaceTilt * normal);
          vec4 mvPosition = vec4(center.xyz + uFaceTilt * position * nodeScale, 1.0);
        #else
          vNodeNormal = normalize(normalMatrix * mat3(instanceMatrix) * normal);
          vec4 mvPosition = modelViewMatrix * instanceMatrix * vec4(position, 1.0);
        #endif
        gl_Position = projectionMatrix * mvPosition;
        #include <fog_vertex>
      }
    `,
    fragmentShader: `
      #include <common>
      #include <fog_pars_fragment>
      varying vec3 vNodeColor;
      varying vec3 vNodeNormal;
      uniform float uDiffuseShare;
      uniform float uNodeEmissiveShare;
      uniform vec3 uKeyColor;
      uniform vec3 uFillColor;
      uniform vec3 uKeyDirection;
      uniform vec3 uFillDirection;
      uniform float uKeyIntensity;
      uniform float uFillIntensity;
      uniform float uAmbientIntensity;
      void main() {
        vec3 normal = normalize(vNodeNormal);
        float key = max(dot(normal, normalize(uKeyDirection)), 0.0) * uKeyIntensity;
        float fill = max(dot(normal, normalize(uFillDirection)), 0.0) * uFillIntensity;
        vec3 lambert = (vec3(uAmbientIntensity) + uKeyColor * key + uFillColor * fill) * uDiffuseShare;
        vec3 outgoingLight = vNodeColor * (lambert + vec3(uNodeEmissiveShare));
        gl_FragColor = vec4(outgoingLight, 1.0);
        #include <tonemapping_fragment>
        #include <colorspace_fragment>
        #include <fog_fragment>
      }
    `,
  });
}

function meshForShape(shape: ShapeName, count: number): InstancedMesh {
  const faceCamera = shape === "torus";
  const mesh = new InstancedMesh(shapeGeometry(shape), material(faceCamera), count);
  mesh.instanceMatrix.setUsage(DynamicDrawUsage);
  mesh.instanceColor?.setUsage(DynamicDrawUsage);
  if (faceCamera) {
    // The world-space geometry no longer matches what is drawn; pick against
    // the ring's bounding sphere, which covers every drawn orientation.
    mesh.raycast = instanceSphereRaycast(mesh, TORUS_OUTER_RADIUS);
  }
  return mesh;
}

const RAY_MATRIX = new Matrix4();
const RAY_SPHERE = new Sphere();
const RAY_POINT = new Vector3();
const RAY_SCALE = new Vector3();

export function instanceSphereRaycast(
  mesh: InstancedMesh,
  radiusFactor: number,
): (raycaster: Raycaster, intersects: Intersection[]) => void {
  return (raycaster, intersects) => {
    for (let instanceId = 0; instanceId < mesh.count; instanceId += UNIT) {
      mesh.getMatrixAt(instanceId, RAY_MATRIX);
      RAY_MATRIX.premultiply(mesh.matrixWorld);
      RAY_SCALE.setFromMatrixScale(RAY_MATRIX);
      RAY_SPHERE.center.setFromMatrixPosition(RAY_MATRIX);
      RAY_SPHERE.radius = RAY_SCALE.x * radiusFactor;
      if (raycaster.ray.intersectSphere(RAY_SPHERE, RAY_POINT) === null) {
        continue;
      }
      const distance = raycaster.ray.origin.distanceTo(RAY_POINT);
      if (distance < raycaster.near || distance > raycaster.far) {
        continue;
      }
      intersects.push({ distance, point: RAY_POINT.clone(), object: mesh, instanceId });
    }
  };
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
