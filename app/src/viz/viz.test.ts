import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";
import {
  Color,
  FogExp2,
  InstancedMesh,
  Matrix4,
  Mesh,
  Object3D,
  PerspectiveCamera,
  Raycaster,
  ShaderMaterial,
  Vector3,
  type Intersection,
} from "three";
import type { ProjectionDto } from "../state/state";
import type { Theme } from "../theme/tokens";
import { fitFrame, motionSettings } from "./camera";
import { RENDER_TOKENS } from "./config";
import { buildEdgeBuffers, buildEdgeLayer, expectedEdgeVertexCount, weightToAlpha } from "./edges";
import { buildHaloLayer, selectHaloCandidates } from "./halos";
import {
  labelCandidates,
  labelScaleForDistance,
  selectVisibleLabels,
  truncateLabel,
  visibleLabelCap,
  type LabelCandidate,
} from "./labels";
import { computeLayout, layoutSnapshot } from "./layout";
import { buildNodeLayer, degreeToScale, degreesForProjection, recolorNodeLayer } from "./nodes";
import {
  dispatchHoveredEntity,
  dispatchPickedEntity,
  entityIdFromIntersection,
  PickingController,
  type PickDispatch,
} from "./picking";
import { effectivePixelRatio, profileForTier, QualityManager } from "./quality";
import { createVizScene } from "./scene";

const entityA = "entity-a";
const entityB = "entity-b";
const entityC = "entity-c";
const repo = fileURLToPath(new URL("../../..", import.meta.url));
const generated = [
  path.join(repo, "fixtures/groups/research-network.ops.jsonl"),
  path.join(repo, "fixtures/groups/fisheries-committee.ops.jsonl"),
];

function projection(): ProjectionDto {
  return {
    revision: 1,
    entities: [
      { id: entityA, kind: "person", attributes: {} },
      { id: entityB, kind: "place", attributes: {} },
      { id: entityC, kind: "person", attributes: {} },
    ],
    edges: [
      { id: "edge-a", from: entityA, to: entityB, weight: 5 },
      { id: "edge-b", from: entityB, to: entityC, weight: 0 },
    ],
  };
}

function theme(person = "#ff0000", place = "#00ff00"): Theme {
  return {
    schema_version: "0.1.0",
    tokens: {
      "kind.person.base": { hex: person, source: "template" },
      "kind.place.base": { hex: place, source: "template" },
    },
  };
}

function kindMeta() {
  return {
    person: { shape: "sphere" as const, label: "Person", colorRole: "kind-1" },
    place: { shape: "cube" as const, label: "Place", colorRole: "kind-2" },
  };
}

function hashFiles(): readonly string[] {
  return generated.map((file) => createHash("sha256").update(readFileSync(file)).digest("hex"));
}

describe("layout", () => {
  it("is deterministic and independent of entity order", () => {
    const entities = projection().entities ?? [];
    const forward = layoutSnapshot(computeLayout(entities));
    const reversed = layoutSnapshot(computeLayout([...entities].reverse()));

    expect(layoutSnapshot(computeLayout(entities))).toEqual(forward);
    expect(reversed).toEqual(forward);
  });

  it("keeps same-kind entities closer together than entities of different kinds", () => {
    const kinds = ["person", "organization", "place", "skill", "need", "project"];
    const entities = kinds.flatMap((kind) =>
      Array.from({ length: 20 }, (_, index) => ({ id: `${kind}-${index}`, kind, attributes: {} })),
    );
    const positions = computeLayout(entities).positions;
    const same: number[] = [];
    const cross: number[] = [];
    for (const [leftIndex, left] of entities.entries()) {
      for (const right of entities.slice(leftIndex + 1)) {
        const distance = positions.get(left.id)?.distanceTo(positions.get(right.id) ?? new Vector3()) ?? 0;
        (left.kind === right.kind ? same : cross).push(distance);
      }
    }
    const median = (values: number[]): number => [...values].sort((a, b) => a - b)[Math.floor(values.length / 2)] ?? 0;

    expect(median(same)).toBeLessThan(median(cross) * 0.6);
  });
});

describe("demo generator", () => {
  it("is byte-stable across two runs", () => {
    execFileSync("node", ["scripts/generate-demo-ops.mjs"], { cwd: path.join(repo, "app") });
    const first = hashFiles();
    execFileSync("node", ["scripts/generate-demo-ops.mjs"], { cwd: path.join(repo, "app") });

    expect(hashFiles()).toEqual(first);
  });
});

describe("nodes", () => {
  it("round-trips instance mappings and recolors without changing counts", () => {
    const data = projection();
    const layer = buildNodeLayer({
      projection: data,
      layout: computeLayout(data.entities ?? []),
      kindMeta: kindMeta(),
      theme: theme(),
      degrees: degreesForProjection(data),
    });
    const before = layer.records.map((record) => record.mesh.count);
    recolorNodeLayer(layer, data, theme("#0000ff", "#ffff00"));

    expect(layer.entityToMesh.get(entityA)?.instanceId).toBe(0);
    const entityBMesh = layer.entityToMesh.get(entityB)?.mesh;
    if (entityBMesh === undefined) {
      throw new Error("missing entity-b mesh");
    }
    expect(layer.meshToEntityIds.get(entityBMesh)).toContain(entityB);
    expect(layer.records.map((record) => record.mesh.count)).toEqual(before);
    layer.dispose();
  });

  it("splits per-instance kind color between diffuse and emissive material paths", () => {
    const data = projection();
    const layer = buildNodeLayer({
      projection: data,
      layout: computeLayout(data.entities ?? []),
      kindMeta: kindMeta(),
      theme: theme(),
      degrees: degreesForProjection(data),
    });
    const material = layer.records[0]?.mesh.material;
    if (!(material instanceof ShaderMaterial)) {
      throw new Error("missing node material");
    }
    const diffuseShare = 1 - RENDER_TOKENS.node.emissiveShare;

    expect(material.vertexShader).toContain("vNodeColor = instanceColor");
    expect(material.fragmentShader).toContain("uNodeEmissiveShare");
    expect(material.uniforms.uDiffuseShare?.value).toBeCloseTo(diffuseShare);
    expect(material.uniforms.uNodeEmissiveShare?.value).toBe(RENDER_TOKENS.node.emissiveShare);
    layer.dispose();
  });

  it("turns rings toward the camera and picks them by their bounding sphere", () => {
    const data = projection();
    const layer = buildNodeLayer({
      projection: data,
      layout: computeLayout(data.entities ?? []),
      kindMeta: { person: { shape: "torus" as const, label: "Person", colorRole: "kind-1" }, place: kindMeta().place },
      theme: theme(),
      degrees: degreesForProjection(data),
    });
    const ring = layer.records.find((record) => record.shape === "torus");
    const cube = layer.records.find((record) => record.shape === "cube");
    const target = layer.entityToMesh.get(entityA);
    if (ring === undefined || cube === undefined || target === undefined) {
      throw new Error("missing ring test meshes");
    }
    expect((ring.mesh.material as ShaderMaterial).defines).toHaveProperty("FACE_CAMERA");
    expect((cube.mesh.material as ShaderMaterial).defines).not.toHaveProperty("FACE_CAMERA");

    // A ray straight through the ring's hole misses the world-space torus, but
    // the drawn ring is camera-facing and tilted, so picking uses its sphere.
    const matrix = new Matrix4();
    const center = new Vector3();
    target.mesh.getMatrixAt(target.instanceId, matrix);
    center.setFromMatrixPosition(matrix);
    const raycaster = new Raycaster(center.clone().add(new Vector3(0, 0, 1000)), new Vector3(0, 0, -1));
    const hits = raycaster.intersectObject(ring.mesh, false);

    expect(hits.some((hit) => hit.instanceId === target.instanceId)).toBe(true);
    layer.dispose();
  });

  it("maps degree to bounded scale", () => {
    expect(degreeToScale(-1)).toBe(3);
    expect(degreeToScale(99)).toBe(10);
  });
});

describe("edges", () => {
  it("builds two vertices per edge with endpoint colors and clamped alpha", () => {
    const data = projection();
    const buffers = buildEdgeBuffers(data, computeLayout(data.entities ?? []), theme());
    const firstColor = new Color("#ff0000");

    expect(buffers.positions.length).toBe(expectedEdgeVertexCount(data.edges?.length ?? 0) * 3);
    expect(buffers.colors.length).toBe(expectedEdgeVertexCount(data.edges?.length ?? 0) * 4);
    expect(buffers.colors[0] ?? Number.NaN).toBeCloseTo(firstColor.r);
    expect(buffers.colors[3] ?? Number.NaN).toBeCloseTo(weightToAlpha(5));
    expect(weightToAlpha(-99)).toBe(0.08);
    expect(weightToAlpha(99)).toBe(0.6);
  });
});

describe("render pipeline agreement", () => {
  it("puts nodes, edges, and halos through the same fog and output chunks", () => {
    const data = projection();
    const layout = computeLayout(data.entities ?? []);
    const nodes = buildNodeLayer({ projection: data, layout, kindMeta: kindMeta(), theme: theme(), degrees: degreesForProjection(data) });
    const edges = buildEdgeLayer(data, layout, theme());
    const halos = buildHaloLayer({
      projection: data,
      layout,
      kindMeta: kindMeta(),
      theme: theme(),
      tier: "A",
      cameraPosition: new Vector3(0, 0, 0),
      viewMode: "overview",
    });
    const materials = [
      nodes.records[0]?.mesh.material,
      edges.object.material,
      (halos.field.children[0] as InstancedMesh | undefined)?.material,
      halos.selected.material,
    ];
    for (const material of materials) {
      if (!(material instanceof ShaderMaterial)) {
        throw new Error("expected a shader material");
      }
      expect(material.fog).toBe(true);
      expect(material.uniforms.fogDensity).toBeDefined();
      expect(material.vertexShader).toContain("#include <fog_vertex>");
      for (const chunk of ["tonemapping_fragment", "colorspace_fragment", "fog_fragment"]) {
        expect(material.fragmentShader).toContain(`#include <${chunk}>`);
      }
    }
    nodes.dispose();
    edges.dispose();
    halos.dispose();
  });
});

describe("halos", () => {
  it("keeps resting halos visible across the small default scene and scales shells around nodes", () => {
    const data = projection();
    const layout = computeLayout(data.entities ?? []);
    const layer = buildHaloLayer({
      projection: data,
      layout,
      kindMeta: kindMeta(),
      theme: theme(),
      tier: "B",
      cameraPosition: new Vector3(0, 0, RENDER_TOKENS.camera.initialZ),
      viewMode: "overview",
    });
    const haloCount = layer.field.children.reduce(
      (sum, child) => sum + (child instanceof InstancedMesh ? child.count : 0),
      0,
    );
    const firstMesh = layer.field.children[0];
    if (!(firstMesh instanceof InstancedMesh)) {
      throw new Error("missing halo mesh");
    }
    const matrix = new Matrix4();
    const decomposed = new Object3D();
    firstMesh.getMatrixAt(0, matrix);
    matrix.decompose(decomposed.position, decomposed.quaternion, decomposed.scale);

    expect(haloCount).toBe(data.entities?.length);
    expect(decomposed.scale.x).toBeGreaterThan(degreeToScale(1));
    layer.dispose();
  });

  it("applies the tier distance before the cap, and follows the camera on refresh", () => {
    const near = new Vector3(0, 0, 0);
    const far = new Vector3(0, 0, RENDER_TOKENS.halo.tierBDistance + 100);
    const picked = selectHaloCandidates([far, near], new Vector3(0, 0, 0), "B");

    expect(picked.map((candidate) => candidate.entryIndex)).toEqual([1]);
    expect(selectHaloCandidates([near], near, "C")).toEqual([]);

    const data = projection();
    const layout = computeLayout(data.entities ?? []);
    const layer = buildHaloLayer({
      projection: data,
      layout,
      kindMeta: kindMeta(),
      theme: theme(),
      tier: "B",
      cameraPosition: new Vector3(0, 0, 100_000),
      viewMode: "overview",
    });
    const visible = (): number => layer.field.children.reduce(
      (sum, child) => sum + (child.visible && child instanceof InstancedMesh ? child.count : 0),
      0,
    );
    expect(visible()).toBe(0);
    layer.refresh(new Vector3(0, 0, 0));
    expect(visible()).toBe(data.entities?.length);
    layer.setSelected(entityA);
    expect(layer.selected.visible).toBe(true);
    layer.setSelected(null);
    expect(layer.selected.visible).toBe(false);
    layer.dispose();
  });
});

describe("labels", () => {
  const origin = new Vector3(0, 0, 0);

  function candidate(id: string, distance: number, degree: number): LabelCandidate {
    return { id, text: id, position: new Vector3(distance, 0, 0), degree };
  }

  it("extracts projection display names and truncates long ones with an ascii ellipsis", () => {
    const data: ProjectionDto = {
      revision: 1,
      entities: [
        {
          id: entityA,
          kind: "person",
          attributes: { display_name: { type: "text", value: "Synthetic Researcher One" } },
        },
        {
          id: entityB,
          kind: "place",
          attributes: {
            display_name: { type: "text", value: "An Extremely Long Synthetic Fixture Location Name" },
          },
        },
      ],
    };
    const layout = computeLayout(data.entities ?? []);
    const candidates = labelCandidates(data, layout, kindMeta(), new Map([[entityA, 2]]));

    expect(candidates.map((entry) => entry.id)).toEqual([entityA, entityB]);
    expect(candidates[0]?.text).toBe("Synthetic Researcher One");
    expect(candidates[0]?.degree).toBe(2);
    expect(candidates[1]?.text.length).toBeLessThanOrEqual(RENDER_TOKENS.label.maxChars);
    expect(candidates[1]?.text.endsWith("...")).toBe(true);
    expect(truncateLabel("short")).toBe("short");
  });

  it("density-culls to a per-tier cap and never renders all labels at pilot scale", () => {
    const pilotScale = 1500;
    const near = Array.from({ length: pilotScale }, (_, index) => candidate(`entity-${index}`, 10, 0));
    for (const tier of ["A", "B", "C", "D"] as const) {
      const visible = selectVisibleLabels(near, origin, tier);
      expect(visible.length).toBe(visibleLabelCap(tier));
      expect(visible.length).toBeLessThan(pilotScale);
    }
    expect(visibleLabelCap("D")).toBeLessThan(visibleLabelCap("C"));
    expect(visibleLabelCap("C")).toBeLessThan(visibleLabelCap("B"));
    expect(visibleLabelCap("B")).toBeLessThan(visibleLabelCap("A"));
    expect(visibleLabelCap("D", "present")).toBe(RENDER_TOKENS.label.capPresent);
  });

  it("is zoom-adaptive: near leaves label, far leaves do not, hubs label from farther out", () => {
    const nearLeaf = candidate("near-leaf", RENDER_TOKENS.label.visibleDistance - 50, 0);
    const farLeaf = candidate("far-leaf", RENDER_TOKENS.label.visibleDistance * 2, 0);
    const farHub = candidate("far-hub", RENDER_TOKENS.label.visibleDistance * 2, RENDER_TOKENS.node.referenceDegree);
    const visible = selectVisibleLabels([nearLeaf, farLeaf, farHub], origin, "A");

    expect(visible.map((entry) => entry.id)).toContain("near-leaf");
    expect(visible.map((entry) => entry.id)).toContain("far-hub");
    expect(visible.map((entry) => entry.id)).not.toContain("far-leaf");
  });

  it("holds on-screen label size once the camera is inside the full-size distance", () => {
    const full = RENDER_TOKENS.label.fullSizeDistance;
    expect(labelScaleForDistance(full * 3)).toBe(1);
    expect(labelScaleForDistance(full)).toBe(1);
    expect(labelScaleForDistance(full / 2)).toBeCloseTo(0.5);
    expect(labelScaleForDistance(-1)).toBe(0);
  });

  it("labels an emphasized set regardless of the zoom distance, still under the cap", () => {
    const far = Array.from({ length: 60 }, (_, index) =>
      candidate(`far-${index}`, RENDER_TOKENS.label.visibleDistance * 3, 0));

    expect(selectVisibleLabels(far, origin, "A")).toEqual([]);
    expect(selectVisibleLabels(far, origin, "A", "overview", true).length).toBe(visibleLabelCap("A"));
  });

  it("orders deterministically by adjusted distance with id tie-break", () => {
    const tied = [candidate("b-tied", 100, 0), candidate("a-tied", 100, 0), candidate("closer", 50, 0)];
    const visible = selectVisibleLabels(tied, origin, "A");

    expect(visible.map((entry) => entry.id)).toEqual(["closer", "a-tied", "b-tied"]);
  });
});

describe("scene", () => {
  it("creates a non-occluding gradient background quad with fog from theme tokens", () => {
    const setup = createVizScene(theme());
    const background = setup.scene.children.find((child) => child instanceof Mesh && child.material instanceof ShaderMaterial);
    if (!(background instanceof Mesh) || !(background.material instanceof ShaderMaterial)) {
      throw new Error("missing background mesh");
    }

    expect(background.geometry.type).toBe("PlaneGeometry");
    expect(background.frustumCulled).toBe(false);
    expect(background.material.depthTest).toBe(false);
    expect(background.material.fragmentShader).toContain("colorspace_fragment");
    expect(setup.scene.fog?.color.getHexString()).toBe("06080d");
    if (!(setup.scene.fog instanceof FogExp2)) {
      throw new Error("missing exponential fog");
    }
    setup.setPresentMode(true);
    expect(setup.scene.fog.density).toBe(RENDER_TOKENS.scene.fogDensity * 0.5);
    setup.setPresentMode(false);
    expect(setup.scene.fog.density).toBe(RENDER_TOKENS.scene.fogDensity);
    setup.dispose();
  });
});

describe("quality", () => {
  it("degrades halos first and upgrades with hysteresis", () => {
    const manager = new QualityManager();
    for (let index = 0; index < 90; index += 1) {
      manager.sample(100);
    }
    expect(manager.profile).toEqual(profileForTier("C"));
    for (let index = 0; index < 360; index += 1) {
      manager.sample(1);
    }
    expect(manager.profile).toEqual(profileForTier("B"));
  });
});

describe("picking hover", () => {
  it("re-dispatches hover after the state machine clears it, and clears on pointercancel", () => {
    const data = projection();
    const layer = buildNodeLayer({
      projection: data,
      layout: computeLayout(data.entities ?? []),
      kindMeta: kindMeta(),
      theme: theme(),
      degrees: degreesForProjection(data),
    });
    const target = layer.entityToMesh.get(entityA);
    if (target === undefined) {
      throw new Error("missing hover target");
    }
    const handlers = new Map<string, (event: unknown) => void>();
    const element = {
      style: { cursor: "" },
      addEventListener: (type: string, handler: (event: unknown) => void) => handlers.set(type, handler),
      removeEventListener: (type: string) => handlers.delete(type),
      getBoundingClientRect: () => ({ left: 0, top: 0, width: 100, height: 100 }),
    };
    let hovered: string | null = null;
    const actions: unknown[] = [];
    const store = {
      dispatch: (action: { kind: string; entityId?: string | null }) => {
        actions.push(action);
        hovered = action.entityId ?? null;
      },
      getState: () => ({ view: { hoveredEntityId: hovered } }),
    };
    const hit = { object: target.mesh, instanceId: target.instanceId, distance: 1 } as unknown as Intersection;
    const originalRaf = globalThis.requestAnimationFrame;
    const queued: FrameRequestCallback[] = [];
    globalThis.requestAnimationFrame = ((callback: FrameRequestCallback) => {
      queued.push(callback);
      return queued.length;
    }) as typeof requestAnimationFrame;
    const moveAndFlush = (event: unknown): void => {
      handlers.get("pointermove")?.(event);
      queued.splice(0).forEach((callback) => callback(0));
    };
    try {
      const picking = new PickingController(
        element as unknown as HTMLElement,
        new PerspectiveCamera(),
        () => layer,
        store as unknown as ConstructorParameters<typeof PickingController>[3],
        () => [hit],
      );
      const move = { clientX: 50, clientY: 50 };
      moveAndFlush(move);
      moveAndFlush(move);
      expect(actions).toHaveLength(1);

      hovered = null; // e.g. entityFocused: the reducer clears hover
      moveAndFlush(move);
      expect(actions).toHaveLength(2);
      expect(hovered).toBe(entityA);

      handlers.get("pointercancel")?.({});
      expect(hovered).toBeNull();
      expect(element.style.cursor).toBe("");
      picking.dispose();
    } finally {
      globalThis.requestAnimationFrame = originalRaf;
      layer.dispose();
    }
  });
});

describe("pixel ratio", () => {
  it("treats the tier DPR as a cap, never an upscale", () => {
    expect(effectivePixelRatio(1, 1.5)).toBe(1);
    expect(effectivePixelRatio(2, 1.5)).toBe(1.5);
    expect(effectivePixelRatio(1.25, 1.25)).toBe(1.25);
    expect(effectivePixelRatio(0, 1.5)).toBe(1);
  });
});

describe("picking", () => {
  it("maps mocked raycast hits and dispatches only store actions", () => {
    const data = projection();
    const layer = buildNodeLayer({
      projection: data,
      layout: computeLayout(data.entities ?? []),
      kindMeta: kindMeta(),
      theme: theme(),
      degrees: degreesForProjection(data),
    });
    const actions: unknown[] = [];
    const store: PickDispatch = { dispatch: (action) => { actions.push(action); } };
    const target = layer.entityToMesh.get(entityB);
    if (target === undefined) {
      throw new Error("missing picking target");
    }
    const hit = {
      object: target.mesh,
      instanceId: target.instanceId,
      distance: 1,
    } as unknown as ReturnType<Raycaster["intersectObject"]>[number];

    expect(entityIdFromIntersection(hit, layer)).toBe(entityB);
    dispatchHoveredEntity(store, entityB);
    dispatchPickedEntity(store, entityB);
    expect(actions).toEqual([
      { kind: "entityHovered", entityId: entityB },
      { kind: "entityFocused", entityId: entityB },
    ]);
    layer.dispose();
  });
});

describe("camera fit", () => {
  const fov = RENDER_TOKENS.camera.fov;
  const towardCamera = new Vector3(0, 0, 1);
  const up = new Vector3(0, 1, 0);

  function projectedInside(points: readonly Vector3[], frame: ReturnType<typeof fitFrame>, aspect: number): boolean {
    const tanV = Math.tan(fov * Math.PI / 360);
    return points.every((point) => {
      const offset = point.clone().sub(frame.target);
      const depthFromCamera = frame.distance - offset.z;
      return depthFromCamera > 0
        && Math.abs(offset.y) / depthFromCamera <= tanV + 1e-9
        && Math.abs(offset.x) / depthFromCamera <= tanV * aspect + 1e-9;
    });
  }

  it("keeps every point inside both FOV axes, including portrait canvases", () => {
    const points = [new Vector3(-400, 0, 0), new Vector3(400, 0, 0), new Vector3(0, 250, 0), new Vector3(0, -250, 200)];
    const landscape = fitFrame(points, towardCamera, up, fov, 16 / 9, 0);
    const portrait = fitFrame(points, towardCamera, up, fov, 9 / 16, 0);

    expect(projectedInside(points, landscape, 16 / 9)).toBe(true);
    expect(projectedInside(points, portrait, 9 / 16)).toBe(true);
    expect(portrait.distance).toBeGreaterThan(landscape.distance * 1.5);
  });

  it("keeps a bottom inset clear by fitting into the window above it", () => {
    const points = [new Vector3(-300, -200, 0), new Vector3(300, 200, 0)];
    const inset = 0.2;
    const frame = fitFrame(points, towardCamera, up, fov, 16 / 9, 0, inset);
    const tanV = Math.tan(fov * Math.PI / 360);
    const ratios = points.map((point) => (point.y - frame.target.y) / frame.distance);

    expect(Math.min(...ratios)).toBeGreaterThanOrEqual(-tanV + 2 * tanV * inset - 1e-9);
    expect(Math.max(...ratios)).toBeLessThanOrEqual(tanV + 1e-9);
  });

  it("frames the screen-plane extent, not a bounding sphere, and centers on it", () => {
    const deepNarrow = [new Vector3(0, 0, -600), new Vector3(0, 0, 600), new Vector3(100, 40, 0), new Vector3(300, -40, 0)];
    const frame = fitFrame(deepNarrow, towardCamera, up, fov, 16 / 9, 0);
    const sphereDistance = 600 / Math.sin(fov * Math.PI / 360);

    expect(projectedInside(deepNarrow, frame, 16 / 9)).toBe(true);
    expect(frame.distance).toBeLessThan(sphereDistance);
    expect(frame.target.x).toBeCloseTo(150);
  });
});

describe("reduced motion", () => {
  it("sets fly-to duration to zero and disables damping drift", () => {
    expect(motionSettings(true)).toEqual({
      durationMs: 0,
      dampingEnabled: false,
      motionScale: 0,
    });
  });
});
