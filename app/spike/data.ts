export const NODE_COUNT = 5_000;
export const EDGE_COUNT = 10_000;

export type SpikeKindId = "sphere" | "cube" | "octahedron" | "tetrahedron" | "torus" | "cone";

export type SpikeNode = {
  id: string;
  kind: SpikeKindId;
  color: string;
  x: number;
  y: number;
  z: number;
  fx: number;
  fy: number;
  fz: number;
  val: number;
};

export type SpikeEdge = {
  id: string;
  source: string;
  target: string;
  weight: number;
};

export type SpikeData = {
  nodes: SpikeNode[];
  links: SpikeEdge[];
};

export type ResolvedSpikeEdge = {
  id: string;
  source: SpikeNode;
  target: SpikeNode;
  weight: number;
};

export const SPIKE_KINDS: ReadonlyArray<{ id: SpikeKindId; color: string }> = [
  { id: "sphere", color: "#e8a87c" },
  { id: "cube", color: "#7ca6e8" },
  { id: "octahedron", color: "#9c8ce8" },
  { id: "tetrahedron", color: "#e8d47c" },
  { id: "torus", color: "#e87ca6" },
  { id: "cone", color: "#7ce8b2" },
];

const SEED = 0x5eed_5000;
const CLUSTER_RADIUS = 600;
const CLUSTER_SPREAD = 115;

class Lcg {
  private state = SEED;

  next(): number {
    this.state = (Math.imul(1_664_525, this.state) + 1_013_904_223) >>> 0;
    return this.state / 0x1_0000_0000;
  }

  range(min: number, max: number): number {
    return min + (max - min) * this.next();
  }

  int(maxExclusive: number): number {
    return Math.floor(this.next() * maxExclusive);
  }
}

export function createSpikeData(): SpikeData {
  const rng = new Lcg();
  const centers = SPIKE_KINDS.map((_, index) => clusterCenter(index));
  const nodes = Array.from({ length: NODE_COUNT }, (_, index) => {
    const kindIndex = index % SPIKE_KINDS.length;
    const kind = SPIKE_KINDS[kindIndex];
    const center = centers[kindIndex];

    if (kind === undefined || center === undefined) {
      throw new Error(`Missing deterministic kind or cluster center for index ${index}`);
    }

    const offset = sphericalOffset(rng, CLUSTER_SPREAD);
    const x = center.x + offset.x;
    const y = center.y + offset.y;
    const z = center.z + offset.z;

    return {
      id: `node-${index}`,
      kind: kind.id,
      color: kind.color,
      x,
      y,
      z,
      fx: x,
      fy: y,
      fz: z,
      val: 4,
    };
  });

  const links: SpikeEdge[] = [];
  const used = new Set<string>();
  const hubEdges = EDGE_COUNT / 4;
  const ringEdges = EDGE_COUNT / 4;

  for (let i = 0; i < hubEdges; i += 1) {
    pushEdge(links, used, 0, 1 + (i % (NODE_COUNT - 1)), rng);
  }

  for (let i = 0; i < ringEdges; i += 1) {
    pushEdge(links, used, i % NODE_COUNT, (i + 1) % NODE_COUNT, rng);
  }

  while (links.length < EDGE_COUNT) {
    const source = rng.int(NODE_COUNT);
    let target = rng.int(NODE_COUNT - 1);
    if (target >= source) {
      target += 1;
    }
    pushEdge(links, used, source, target, rng);
  }

  return { nodes, links };
}

export function resolveEdges(data: SpikeData): ResolvedSpikeEdge[] {
  const nodesById = new Map(data.nodes.map((node) => [node.id, node]));

  return data.links.map((link) => {
    const source = nodesById.get(link.source);
    const target = nodesById.get(link.target);
    if (source === undefined || target === undefined) {
      throw new Error(`Invalid spike edge endpoint for ${link.id}`);
    }
    return { ...link, source, target };
  });
}

function clusterCenter(index: number): { x: number; y: number; z: number } {
  const phi = Math.acos(1 - (2 * (index + 0.5)) / SPIKE_KINDS.length);
  const theta = Math.PI * (1 + Math.sqrt(5)) * (index + 0.5);

  return {
    x: CLUSTER_RADIUS * Math.sin(phi) * Math.cos(theta),
    y: CLUSTER_RADIUS * Math.cos(phi),
    z: CLUSTER_RADIUS * Math.sin(phi) * Math.sin(theta),
  };
}

function sphericalOffset(rng: Lcg, radius: number): { x: number; y: number; z: number } {
  const u = rng.range(-1, 1);
  const theta = rng.range(0, Math.PI * 2);
  const r = radius * Math.cbrt(rng.next());
  const xy = Math.sqrt(1 - u * u);

  return {
    x: r * xy * Math.cos(theta),
    y: r * u,
    z: r * xy * Math.sin(theta),
  };
}

function pushEdge(edges: SpikeEdge[], used: Set<string>, source: number, target: number, rng: Lcg): void {
  const low = Math.min(source, target);
  const high = Math.max(source, target);
  const key = `${low}:${high}`;
  if (used.has(key)) {
    return;
  }
  used.add(key);
  edges.push({
    id: `edge-${edges.length}`,
    source: `node-${source}`,
    target: `node-${target}`,
    weight: Number(rng.next().toFixed(4)),
  });
}
