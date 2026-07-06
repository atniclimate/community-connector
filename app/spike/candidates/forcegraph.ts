import ThreeForceGraph from "three-forcegraph";
import * as THREE from "three";
import type { SpikeData, SpikeEdge, SpikeNode } from "../data";
import type { CandidateHandle } from "../harness";

type ThreeGraph = ThreeForceGraph<SpikeNode, SpikeEdge>;

export function createForceGraphCandidate(container: HTMLElement, data: SpikeData): CandidateHandle {
  const scene = new THREE.Scene();
  scene.background = new THREE.Color("#070a10");
  scene.fog = new THREE.FogExp2("#070a10", 0.0007);

  const camera = new THREE.PerspectiveCamera(55, 1, 1, 5_000);
  const renderer = new THREE.WebGLRenderer({ antialias: true, powerPreference: "high-performance" });
  renderer.setPixelRatio(Math.min(window.devicePixelRatio, 1.5));
  container.append(renderer.domElement);

  const graph = new ThreeForceGraph<SpikeNode, SpikeEdge>() as ThreeGraph;
  graph
    .cooldownTicks(0)
    .nodeId("id")
    .nodeVal("val")
    .nodeColor("color")
    .nodeResolution(6)
    .linkOpacity(0.22)
    .linkWidth((link: SpikeEdge) => 0.25 + link.weight * 1.25)
    .linkColor((link: SpikeEdge) => edgeColor(link.weight))
    .graphData(cloneGraphData(data));

  scene.add(graph);
  scene.add(new THREE.AmbientLight("#ffffff", 1.4));
  const light = new THREE.DirectionalLight("#ffffff", 1.8);
  light.position.set(500, 700, 900);
  scene.add(light);

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
      graph.tickFrame();
      renderer.render(scene, camera);
    },
    dispose(): void {
      window.removeEventListener("resize", resize);
      renderer.dispose();
      renderer.domElement.remove();
    },
  };
}

function cloneGraphData(data: SpikeData): SpikeData {
  return {
    nodes: data.nodes.map((node) => ({ ...node })),
    links: data.links.map((link) => ({ ...link })),
  };
}

function edgeColor(weight: number): string {
  return `rgba(185, 199, 255, ${0.08 + weight * 0.52})`;
}
