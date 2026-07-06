import ForceGraph3D, { type ForceGraph3DInstance } from "3d-force-graph";
import type { CandidateHandle } from "../harness";
import type { SpikeData, SpikeEdge, SpikeNode } from "../data";
import * as THREE from "three";

type StockGraph = ForceGraph3DInstance<SpikeNode, SpikeEdge>;

export function createStockCandidate(container: HTMLElement, data: SpikeData): CandidateHandle {
  // 3d-force-graph exposes a Kapsule constructor whose boundary is less precise than its chainable instance type.
  const graph = new ForceGraph3D(container, { controlType: "orbit" }) as unknown as StockGraph;

  graph
    .backgroundColor("#070a10")
    .showNavInfo(false)
    .enableNavigationControls(false)
    .enableNodeDrag(false)
    .enablePointerInteraction(false)
    .cooldownTicks(0)
    .nodeId("id")
    .nodeVal("val")
    .nodeColor("color")
    .nodeResolution(6)
    .nodeLabel("")
    .linkLabel("")
    .linkOpacity(0.22)
    .linkWidth((link) => 0.25 + link.weight * 1.25)
    .linkColor((link) => edgeColor(link.weight))
    .graphData(cloneGraphData(data));

  const resize = (): void => {
    graph.width(container.clientWidth).height(container.clientHeight);
  };
  resize();
  window.addEventListener("resize", resize);

  const renderer = graph.renderer();
  renderer.setPixelRatio(Math.min(window.devicePixelRatio, 1.5));

  return {
    renderer,
    setCamera(position: THREE.Vector3, lookAt: THREE.Vector3): void {
      graph.cameraPosition({ x: position.x, y: position.y, z: position.z }, lookAt, 0);
    },
    dispose(): void {
      window.removeEventListener("resize", resize);
      graph._destructor();
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
