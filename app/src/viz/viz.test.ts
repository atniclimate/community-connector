import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";
import {
  Color,
  InstancedMesh,
  Matrix4,
  Mesh,
  Object3D,
  Raycaster,
  ShaderMaterial,
  Vector3,
} from "three";
import type { ProjectionDto } from "../state/state";
import type { Theme } from "../theme/tokens";
import { motionSettings } from "./camera";
import { RENDER_TOKENS } from "./config";
import { buildEdgeBuffers, expectedEdgeVertexCount, weightToAlpha } from "./edges";
import { buildHaloLayer } from "./halos";
import {
  labelCandidates,
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
  type PickDispatch,
} from "./picking";
import { profileForTier, QualityManager } from "./quality";
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
    });
    const haloCount = layer.group.children.reduce(
      (sum, child) => sum + (child instanceof InstancedMesh ? child.count : 0),
      0,
    );
    const firstMesh = layer.group.children[0];
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

describe("reduced motion", () => {
  it("sets fly-to duration to zero and disables damping drift", () => {
    expect(motionSettings(true)).toEqual({
      durationMs: 0,
      dampingEnabled: false,
      motionScale: 0,
    });
  });
});
