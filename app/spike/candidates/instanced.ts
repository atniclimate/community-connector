import * as THREE from "three";
import { resolveEdges, SPIKE_KINDS, type SpikeData, type SpikeKindId, type SpikeNode } from "../data";
import type { CandidateHandle } from "../harness";

const NODE_SIZE = 11;
const HALO_SCALE = 1.85;

export function createInstancedCandidate(container: HTMLElement, data: SpikeData): CandidateHandle {
  const scene = new THREE.Scene();
  scene.background = new THREE.Color("#070a10");
  scene.fog = new THREE.FogExp2("#070a10", 0.0008);

  const camera = new THREE.PerspectiveCamera(55, 1, 1, 5_000);
  const renderer = new THREE.WebGLRenderer({ antialias: true, powerPreference: "high-performance" });
  renderer.setPixelRatio(Math.min(window.devicePixelRatio, 1.5));
  container.append(renderer.domElement);

  scene.add(new THREE.AmbientLight("#ffffff", 1.25));
  const light = new THREE.DirectionalLight("#ffffff", 1.7);
  light.position.set(600, 800, 900);
  scene.add(light);

  const geometryByKind = createGeometryByKind();
  for (const kind of SPIKE_KINDS) {
    const nodes = data.nodes.filter((node) => node.kind === kind.id);
    const geometry = geometryByKind.get(kind.id);
    if (geometry === undefined) {
      throw new Error(`Missing instanced geometry for ${kind.id}`);
    }
    scene.add(createNodeMesh(geometry, nodes));
    scene.add(createHaloMesh(nodes));
  }
  scene.add(createEdgeSegments(data));

  const resize = (): void => {
    const width = container.clientWidth;
    const height = container.clientHeight;
    camera.aspect = width / Math.max(height, 1);
    camera.updateProjectionMatrix();
    renderer.setSize(width, height, false);
  };
  resize();
  window.addEventListener("resize", resize);

  return {
    renderer,
    setCamera(position: THREE.Vector3, lookAt: THREE.Vector3): void {
      camera.position.copy(position);
      camera.lookAt(lookAt);
    },
    render(): void {
      renderer.render(scene, camera);
    },
    dispose(): void {
      window.removeEventListener("resize", resize);
      scene.traverse((object) => {
        if (object instanceof THREE.Mesh || object instanceof THREE.LineSegments) {
          object.geometry.dispose();
          const material = object.material;
          if (Array.isArray(material)) {
            for (const item of material) {
              item.dispose();
            }
          } else {
            material.dispose();
          }
        }
      });
      renderer.dispose();
      renderer.domElement.remove();
    },
  };
}

function createGeometryByKind(): Map<SpikeKindId, THREE.BufferGeometry> {
  return new Map<SpikeKindId, THREE.BufferGeometry>([
    ["sphere", new THREE.IcosahedronGeometry(NODE_SIZE, 1)],
    ["cube", new THREE.BoxGeometry(NODE_SIZE * 1.55, NODE_SIZE * 1.55, NODE_SIZE * 1.55, 1, 1, 1)],
    ["octahedron", new THREE.OctahedronGeometry(NODE_SIZE * 1.35, 0)],
    ["tetrahedron", new THREE.TetrahedronGeometry(NODE_SIZE * 1.55, 0)],
    ["torus", new THREE.TorusGeometry(NODE_SIZE * 0.92, NODE_SIZE * 0.28, 6, 12)],
    ["cone", new THREE.ConeGeometry(NODE_SIZE * 0.95, NODE_SIZE * 2.1, 8, 1)],
  ]);
}

function createNodeMesh(geometry: THREE.BufferGeometry, nodes: SpikeNode[]): THREE.InstancedMesh {
  const material = new THREE.MeshLambertMaterial({ vertexColors: true });
  const mesh = new THREE.InstancedMesh(geometry, material, nodes.length);
  const matrix = new THREE.Matrix4();
  const color = new THREE.Color();

  nodes.forEach((node, index) => {
    matrix.makeTranslation(node.x, node.y, node.z);
    mesh.setMatrixAt(index, matrix);
    mesh.setColorAt(index, color.set(node.color));
  });
  mesh.instanceMatrix.needsUpdate = true;
  if (mesh.instanceColor !== null) {
    mesh.instanceColor.needsUpdate = true;
  }
  return mesh;
}

function createHaloMesh(nodes: SpikeNode[]): THREE.InstancedMesh {
  const geometry = new THREE.IcosahedronGeometry(NODE_SIZE * HALO_SCALE, 1);
  const material = new THREE.ShaderMaterial({
    transparent: true,
    depthWrite: false,
    side: THREE.BackSide,
    blending: THREE.AdditiveBlending,
    vertexColors: true,
    vertexShader: `
      varying vec3 vNormal;
      varying vec3 vColor;
      void main() {
        vNormal = normalize(normalMatrix * normal);
        vColor = instanceColor;
        gl_Position = projectionMatrix * modelViewMatrix * instanceMatrix * vec4(position, 1.0);
      }
    `,
    fragmentShader: `
      varying vec3 vNormal;
      varying vec3 vColor;
      void main() {
        float rim = pow(1.0 - abs(vNormal.z), 2.2);
        gl_FragColor = vec4(vColor, rim * 0.18);
      }
    `,
  });
  const mesh = new THREE.InstancedMesh(geometry, material, nodes.length);
  const matrix = new THREE.Matrix4();
  const color = new THREE.Color();

  nodes.forEach((node, index) => {
    matrix.makeTranslation(node.x, node.y, node.z);
    mesh.setMatrixAt(index, matrix);
    mesh.setColorAt(index, color.set(node.color));
  });
  mesh.instanceMatrix.needsUpdate = true;
  if (mesh.instanceColor !== null) {
    mesh.instanceColor.needsUpdate = true;
  }
  return mesh;
}

function createEdgeSegments(data: SpikeData): THREE.LineSegments {
  const edges = resolveEdges(data);
  const positions = new Float32Array(edges.length * 6);
  const colors = new Float32Array(edges.length * 6);
  const color = new THREE.Color();

  edges.forEach((edge, index) => {
    const positionOffset = index * 6;
    positions.set([edge.source.x, edge.source.y, edge.source.z, edge.target.x, edge.target.y, edge.target.z], positionOffset);

    writeEdgeColor(colors, positionOffset, color.set(edge.source.color), edge.weight);
    writeEdgeColor(colors, positionOffset + 3, color.set(edge.target.color), edge.weight);
  });

  const geometry = new THREE.BufferGeometry();
  geometry.setAttribute("position", new THREE.BufferAttribute(positions, 3));
  geometry.setAttribute("color", new THREE.BufferAttribute(colors, 3));

  const material = new THREE.LineBasicMaterial({
    vertexColors: true,
    transparent: true,
    opacity: 0.38,
    depthWrite: false,
    blending: THREE.AdditiveBlending,
  });

  return new THREE.LineSegments(geometry, material);
}

function writeEdgeColor(colors: Float32Array, offset: number, color: THREE.Color, weight: number): void {
  const intensity = 0.3 + weight * 0.7;
  colors[offset] = color.r * intensity;
  colors[offset + 1] = color.g * intensity;
  colors[offset + 2] = color.b * intensity;
}
