import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";
import { Color, Raycaster } from "three";
import type { ProjectionDto } from "../state/state";
import type { Theme } from "../theme/tokens";
import { motionSettings } from "./camera";
import { buildEdgeBuffers, expectedEdgeVertexCount, weightToAlpha } from "./edges";
import { computeLayout, layoutSnapshot } from "./layout";
import { buildNodeLayer, degreeToScale, degreesForProjection, recolorNodeLayer } from "./nodes";
import {
  dispatchHoveredEntity,
  dispatchPickedEntity,
  entityIdFromIntersection,
  type PickDispatch,
} from "./picking";
import { profileForTier, QualityManager } from "./quality";

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
