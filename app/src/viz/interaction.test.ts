import { describe, expect, it } from "vitest";
import { BufferAttribute, Color, InstancedMesh, Matrix4, Object3D, ShaderMaterial, Vector3 } from "three";
import { createInitialState, type AppState, type ProjectionDto } from "../state/state";
import { scaleChroma, shiftLightness } from "../theme/color";
import type { Theme, ThemeReport } from "../theme/tokens";
import { driftActive, flightDurationMs } from "./camera";
import { RENDER_TOKENS } from "./config";
import { buildEdgeLayer, setEdgeFocusBlend, weightToAlpha, writeFocusTargetColors } from "./edges";
import {
  applyFocusToNodeLayer,
  computeFocusSet,
  FocusBlend,
  focusRole,
  nodeFocusHex,
  nodeFocusScaleFactor,
} from "./focus";
import { buildHaloLayer, setHaloFocusDim } from "./halos";
import { computeLayout } from "./layout";
import { ADJUSTED_BADGE, buildLegendModel, humanizeEdgeKind, REVIEW_MARKER } from "./legend";
import { buildNodeLayer, degreesForProjection, degreeToScale } from "./nodes";

const entityA = "entity-a";
const entityB = "entity-b";
const entityC = "entity-c";

function projection(): ProjectionDto {
  return {
    revision: 1,
    entities: [
      { id: entityA, kind: "person", attributes: {} },
      { id: entityB, kind: "place", attributes: {} },
      { id: entityC, kind: "person", attributes: {} },
    ],
    edges: [
      { id: "edge-a", kind: "coauthored", from: entityA, to: entityB, weight: 5 },
      { id: "edge-b", kind: "affiliated_with", from: entityB, to: entityC, weight: 0 },
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

function nodeLayer(data: ProjectionDto, themeValue: Theme) {
  return buildNodeLayer({
    projection: data,
    layout: computeLayout(data.entities ?? []),
    kindMeta: kindMeta(),
    theme: themeValue,
    degrees: degreesForProjection(data),
  });
}

function instanceScale(mesh: InstancedMesh, instanceId: number): number {
  const matrix = new Matrix4();
  const decomposed = new Object3D();
  mesh.getMatrixAt(instanceId, matrix);
  matrix.decompose(decomposed.position, decomposed.quaternion, decomposed.scale);
  return decomposed.scale.x;
}

function instanceColor(mesh: InstancedMesh, instanceId: number): Color {
  const color = new Color();
  mesh.getColorAt(instanceId, color);
  return color;
}

function expectColorClose(actual: Color, expectedHex: string): void {
  const expected = new Color(expectedHex);
  expect(actual.r).toBeCloseTo(expected.r);
  expect(actual.g).toBeCloseTo(expected.g);
  expect(actual.b).toBeCloseTo(expected.b);
}

describe("focus set", () => {
  it("collects neighbors and adjacent edges of the focused entity", () => {
    const focus = computeFocusSet(projection(), entityB);
    if (focus === null) {
      throw new Error("missing focus set");
    }

    expect(focus.focusedId).toBe(entityB);
    expect([...focus.neighborIds].sort((a, b) => a.localeCompare(b))).toEqual([entityA, entityC]);
    expect([...focus.adjacentEdgeIds].sort((a, b) => a.localeCompare(b))).toEqual(["edge-a", "edge-b"]);
    expect(computeFocusSet(projection(), null)).toBeNull();
    expect(computeFocusSet(projection(), entityA)?.neighborIds.has(entityC)).toBe(false);
  });

  it("assigns roles and role appearance from theme tokens with derive-matched fallbacks", () => {
    const focus = computeFocusSet(projection(), entityA);

    expect(focusRole(entityA, focus)).toBe("focused");
    expect(focusRole(entityB, focus)).toBe("neighbor");
    expect(focusRole(entityC, focus)).toBe("unrelated");
    expect(focusRole(entityC, null)).toBe("base");
    expect(nodeFocusHex("person", "focused", theme())).toBe(shiftLightness("#ff0000", 0.08));
    expect(nodeFocusHex("person", "unrelated", theme())).toBe(scaleChroma("#ff0000", 0.35));
    expect(nodeFocusHex("person", "neighbor", theme())).toBe("#ff0000");
    const withVariants: Theme = {
      schema_version: "0.1.0",
      tokens: {
        "kind.person.base": { hex: "#ff0000", source: "template" },
        "kind.person.hover": { hex: "#ff8080", source: "template" },
        "kind.person.dim": { hex: "#402020", source: "template" },
      },
    };
    expect(nodeFocusHex("person", "focused", withVariants)).toBe("#ff8080");
    expect(nodeFocusHex("person", "unrelated", withVariants)).toBe("#402020");
    expect(nodeFocusScaleFactor("focused")).toBe(RENDER_TOKENS.node.selectedScale);
    expect(nodeFocusScaleFactor("unrelated")).toBe(RENDER_TOKENS.node.dimmedScale);
    expect(nodeFocusScaleFactor("base")).toBe(1);
  });
});

describe("focus node application", () => {
  it("writes role colors and absolute scales without compounding on re-apply", () => {
    const data = projection();
    const layer = nodeLayer(data, theme());
    const degrees = degreesForProjection(data);
    const focus = computeFocusSet(data, entityA);
    applyFocusToNodeLayer(layer, data, theme(), degrees, focus);
    applyFocusToNodeLayer(layer, data, theme(), degrees, focus);
    const focused = layer.entityToMesh.get(entityA);
    const unrelated = layer.entityToMesh.get(entityC);
    if (focused === undefined || unrelated === undefined) {
      throw new Error("missing focus test instances");
    }
    const focusedBase = degreeToScale(degrees.get(entityA) ?? 0);
    const unrelatedBase = degreeToScale(degrees.get(entityC) ?? 0);

    expect(instanceScale(focused.mesh, focused.instanceId)).toBeCloseTo(
      focusedBase * RENDER_TOKENS.node.selectedScale,
    );
    expect(instanceScale(unrelated.mesh, unrelated.instanceId)).toBeCloseTo(
      Math.max(RENDER_TOKENS.node.minRadius, unrelatedBase * RENDER_TOKENS.node.dimmedScale),
    );
    expectColorClose(instanceColor(focused.mesh, focused.instanceId), shiftLightness("#ff0000", 0.08));
    expectColorClose(instanceColor(unrelated.mesh, unrelated.instanceId), scaleChroma("#ff0000", 0.35));

    applyFocusToNodeLayer(layer, data, theme(), degrees, null);
    expect(instanceScale(focused.mesh, focused.instanceId)).toBeCloseTo(focusedBase);
    expectColorClose(instanceColor(focused.mesh, focused.instanceId), "#ff0000");
    layer.dispose();
  });
});

describe("focus blend", () => {
  it("eases toward the target and reports completion", () => {
    const blend = new FocusBlend();
    blend.setTarget(1);

    expect(blend.update(RENDER_TOKENS.focus.blendMs / 2, false)).toBe(true);
    expect(blend.value).toBeCloseTo(0.5);
    expect(blend.update(RENDER_TOKENS.focus.blendMs, false)).toBe(true);
    expect(blend.value).toBe(1);
    expect(blend.update(16, false)).toBe(false);
  });

  it("snaps in a single update under reduced motion, keeping the end state", () => {
    const blend = new FocusBlend();
    blend.setTarget(1);

    expect(blend.update(1, true)).toBe(true);
    expect(blend.value).toBe(1);
    expect(blend.update(1, true)).toBe(false);
    blend.setTarget(0);
    expect(blend.update(1, true)).toBe(true);
    expect(blend.value).toBe(0);
  });
});

describe("focus edge target colors", () => {
  it("brightens adjacent edges with an alpha floor and dims the rest", () => {
    const data = projection();
    const layer = buildEdgeLayer(data, computeLayout(data.entities ?? []), theme());
    const focus = computeFocusSet(data, entityA);
    writeFocusTargetColors(layer, theme(), focus?.adjacentEdgeIds ?? null);
    const attribute = layer.object.geometry.getAttribute("targetColor") as BufferAttribute;
    const personColor = new Color("#ff0000");
    const placeColor = new Color("#00ff00");

    expect(layer.records.map((record) => record.id)).toEqual(["edge-a", "edge-b"]);
    expect(attribute.getX(0)).toBeCloseTo(personColor.r);
    expect(attribute.getW(0)).toBeCloseTo(Math.max(weightToAlpha(5), RENDER_TOKENS.focus.adjacentMinAlpha));
    expect(attribute.getW(1)).toBeCloseTo(Math.max(weightToAlpha(5), RENDER_TOKENS.focus.adjacentMinAlpha));
    expect(attribute.getY(2)).toBeCloseTo(placeColor.g * RENDER_TOKENS.focus.edgeDimFactor);
    expect(attribute.getW(2)).toBeCloseTo(weightToAlpha(0) * RENDER_TOKENS.focus.edgeDimAlphaFactor);
    // needsUpdate is a write-only setter in three; it bumps version on write.
    expect(attribute.version).toBeGreaterThan(0);

    writeFocusTargetColors(layer, theme(), null);
    expect(attribute.getW(0)).toBeCloseTo(weightToAlpha(5) * RENDER_TOKENS.focus.edgeDimAlphaFactor);
    layer.dispose();
  });

  it("drives the shader blend uniform", () => {
    const data = projection();
    const layer = buildEdgeLayer(data, computeLayout(data.entities ?? []), theme());
    setEdgeFocusBlend(layer, 0.75);

    expect(layer.object.material.uniforms.uFocusBlend?.value).toBe(0.75);
    layer.dispose();
  });
});

describe("focus halo dim", () => {
  it("interpolates halo alpha between resting and dimmed", () => {
    const data = projection();
    const layer = buildHaloLayer({
      projection: data,
      layout: computeLayout(data.entities ?? []),
      kindMeta: kindMeta(),
      theme: theme(),
      tier: "B",
      cameraPosition: new Vector3(0, 0, RENDER_TOKENS.camera.initialZ),
    });
    const alphas = (): readonly number[] =>
      layer.group.children.map((child) => {
        const material = (child as InstancedMesh).material as ShaderMaterial;
        return material.uniforms.uAlpha?.value as number;
      });
    setHaloFocusDim(layer, 1);

    expect(alphas().every((alpha) => Math.abs(alpha - RENDER_TOKENS.halo.restingAlpha * RENDER_TOKENS.focus.haloDimFactor) < 1e-9)).toBe(true);
    setHaloFocusDim(layer, 0);
    expect(alphas().every((alpha) => alpha === RENDER_TOKENS.halo.restingAlpha)).toBe(true);
    layer.dispose();
  });
});

describe("camera motion policy", () => {
  it("selects the near flight duration for short hops", () => {
    expect(flightDurationMs(RENDER_TOKENS.camera.nearFlightDistance)).toBe(RENDER_TOKENS.camera.nearDurationMs);
    expect(flightDurationMs(RENDER_TOKENS.camera.nearFlightDistance + 1)).toBe(
      RENDER_TOKENS.camera.standardDurationMs,
    );
  });

  it("drifts only when enabled, idle past the delay, and motion is allowed", () => {
    const delay = RENDER_TOKENS.drift.idleDelayMs;

    expect(driftActive(true, false, delay, false)).toBe(true);
    expect(driftActive(true, false, delay - 1, false)).toBe(false);
    expect(driftActive(true, true, delay, false)).toBe(false);
    expect(driftActive(false, false, delay, false)).toBe(false);
    expect(driftActive(true, false, delay, true)).toBe(false);
  });
});

describe("legend model", () => {
  function legendState(resolved: Theme | null, report: ThemeReport | null): AppState {
    const base = createInitialState();
    return {
      ...base,
      data: { ...base.data, projection: projection(), detail: null, kindMeta: kindMeta() },
      theme: { resolved, report },
    };
  }

  it("builds kind rows from the live theme with the adjusted indicator and reason", () => {
    const resolved: Theme = {
      schema_version: "0.1.0",
      tokens: {
        "kind.person.base": { hex: "#ff2010", source: "adjusted" },
        "kind.place.base": { hex: "#00ff00", source: "template" },
      },
    };
    const report: ThemeReport = {
      schema_version: "0.1.0",
      adjustments: [
        { token: "kind.person.base", from: "#661008", to: "#ff2010", reason: "WCAG non-text contrast floor" },
      ],
      warnings: [],
    };
    const model = buildLegendModel(legendState(resolved, report));

    expect(model.kinds).toEqual([
      {
        kindId: "person",
        label: "Person",
        shape: "sphere",
        colorHex: "#ff2010",
        adjusted: true,
        adjustedReason: "WCAG non-text contrast floor",
      },
      {
        kindId: "place",
        label: "Place",
        shape: "cube",
        colorHex: "#00ff00",
        adjusted: false,
        adjustedReason: null,
      },
    ]);
  });

  it("lists distinct projection edge kinds, humanized and sorted", () => {
    const model = buildLegendModel(legendState(theme(), null));

    expect(model.edges).toEqual([
      { kindId: "affiliated_with", label: "affiliated with" },
      { kindId: "coauthored", label: "coauthored" },
    ]);
    expect(humanizeEdgeKind("need_met_by")).toBe("need met by");
  });

  it("falls back to the fallback kind color without a theme and pins the quarantine strings", () => {
    const model = buildLegendModel(legendState(null, null));

    expect(model.kinds[0]?.colorHex).toBe("#7cc8e8");
    expect(model.kinds[0]?.adjusted).toBe(false);
    expect(REVIEW_MARKER).toBe("DRAFT - PENDING HUMAN REVIEW (D-023)");
    expect(ADJUSTED_BADGE).toBe("adjusted for readability");
  });
});
