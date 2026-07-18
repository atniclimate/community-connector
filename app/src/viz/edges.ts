import {
  BufferAttribute,
  BufferGeometry,
  Color,
  LineSegments,
  ShaderMaterial,
  Vector3,
} from "three";
import type { ProjectionDto } from "../state/state";
import type { Theme } from "../theme/tokens";
import { RENDER_TOKENS } from "./config";
import type { LayoutResult } from "./layout";
import { kindColor, projectedEdges } from "./projection";

/** One entry per edge actually written into the buffers, in buffer order. */
export type EdgeRenderRecord = {
  readonly id: string;
  readonly fromKind: string;
  readonly toKind: string;
  readonly weight: number;
};

export type EdgeLayer = {
  readonly object: LineSegments<BufferGeometry, ShaderMaterial>;
  readonly records: readonly EdgeRenderRecord[];
  readonly dispose: () => void;
};

const XYZ_COMPONENTS = 3;
const RGBA_COMPONENTS = 4;
const VERTICES_PER_EDGE = 2;
const BASE_BLEND = 0;

export function weightToAlpha(weight: number): number {
  const t = (weight - RENDER_TOKENS.edge.minWeight) /
    (RENDER_TOKENS.edge.maxWeight - RENDER_TOKENS.edge.minWeight);
  const clamped = Math.min(Math.max(t, BASE_BLEND), 1);
  return RENDER_TOKENS.edge.minAlpha +
    clamped * (RENDER_TOKENS.edge.maxAlpha - RENDER_TOKENS.edge.minAlpha);
}

function edgeMaterial(): ShaderMaterial {
  return new ShaderMaterial({
    transparent: true,
    depthWrite: false,
    uniforms: { uFocusBlend: { value: BASE_BLEND } },
    vertexShader: `
      attribute vec4 color;
      attribute vec4 targetColor;
      varying vec4 vColor;
      uniform float uFocusBlend;
      void main() {
        vColor = mix(color, targetColor, uFocusBlend);
        gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0);
      }
    `,
    fragmentShader: `
      varying vec4 vColor;
      void main() {
        gl_FragColor = vColor;
      }
    `,
  });
}

function rgba(color: Color, alpha: number): readonly number[] {
  return [color.r, color.g, color.b, alpha];
}

function targetRgba(color: Color, alpha: number): readonly number[] {
  return [
    color.r * RENDER_TOKENS.focus.edgeDimFactor,
    color.g * RENDER_TOKENS.focus.edgeDimFactor,
    color.b * RENDER_TOKENS.focus.edgeDimFactor,
    alpha * RENDER_TOKENS.focus.edgeDimAlphaFactor,
  ];
}

function highlightRgba(color: Color, alpha: number): readonly number[] {
  return [color.r, color.g, color.b, Math.max(alpha, RENDER_TOKENS.focus.adjacentMinAlpha)];
}

function pushEndpoint(
  target: number[],
  position: Vector3,
  colorValues: readonly number[],
): void {
  target.push(position.x, position.y, position.z, ...colorValues);
}

export function buildEdgeBuffers(
  projection: ProjectionDto,
  layout: LayoutResult,
  theme: Theme | null,
): {
  readonly positions: Float32Array;
  readonly colors: Float32Array;
  readonly targetColors: Float32Array;
  readonly records: readonly EdgeRenderRecord[];
} {
  const positions: number[] = [];
  const colors: number[] = [];
  const targetColors: number[] = [];
  const records: EdgeRenderRecord[] = [];
  const entities = new Map((projection.entities ?? []).map((entity) => [entity.id, entity]));
  for (const edge of projectedEdges(projection)) {
    const from = entities.get(edge.from);
    const to = entities.get(edge.to);
    const fromPosition = layout.positions.get(edge.from);
    const toPosition = layout.positions.get(edge.to);
    if (from === undefined || to === undefined || fromPosition === undefined || toPosition === undefined) {
      continue;
    }
    const alpha = weightToAlpha(edge.weight);
    const fromColor = new Color(kindColor(from.kind ?? "", theme));
    const toColor = new Color(kindColor(to.kind ?? "", theme));
    pushEndpoint(positions, fromPosition, []);
    pushEndpoint(positions, toPosition, []);
    colors.push(...rgba(fromColor, alpha), ...rgba(toColor, alpha));
    targetColors.push(...targetRgba(fromColor, alpha), ...targetRgba(toColor, alpha));
    records.push({ id: edge.id, fromKind: from.kind ?? "", toKind: to.kind ?? "", weight: edge.weight });
  }
  return {
    positions: new Float32Array(positions),
    colors: new Float32Array(colors),
    targetColors: new Float32Array(targetColors),
    records,
  };
}

export function buildEdgeLayer(
  projection: ProjectionDto,
  layout: LayoutResult,
  theme: Theme | null,
): EdgeLayer {
  const buffers = buildEdgeBuffers(projection, layout, theme);
  const geometry = new BufferGeometry();
  geometry.setAttribute("position", new BufferAttribute(buffers.positions, XYZ_COMPONENTS));
  geometry.setAttribute("color", new BufferAttribute(buffers.colors, RGBA_COMPONENTS));
  geometry.setAttribute("targetColor", new BufferAttribute(buffers.targetColors, RGBA_COMPONENTS));
  const material = edgeMaterial();
  const object = new LineSegments(geometry, material);
  return {
    object,
    records: buffers.records,
    dispose: () => {
      geometry.dispose();
      material.dispose();
    },
  };
}

export function expectedEdgeVertexCount(edgeCount: number): number {
  return edgeCount * VERTICES_PER_EDGE;
}

/**
 * Rewrites the focus-target color buffer (P1.2): edges adjacent to the focused
 * entity keep full endpoint colors with a boosted minimum alpha, everything
 * else takes the dimmed variant. With `adjacentEdgeIds` null every edge dims,
 * which matches the buffer's built state.
 */
export function writeFocusTargetColors(
  layer: EdgeLayer,
  theme: Theme | null,
  adjacentEdgeIds: ReadonlySet<string> | null,
): void {
  const attribute = layer.object.geometry.getAttribute("targetColor");
  if (!(attribute instanceof BufferAttribute)) {
    throw new Error("Edge layer is missing its targetColor attribute");
  }
  for (const [index, record] of layer.records.entries()) {
    const alpha = weightToAlpha(record.weight);
    const adjacent = adjacentEdgeIds?.has(record.id) ?? false;
    const fromColor = new Color(kindColor(record.fromKind, theme));
    const toColor = new Color(kindColor(record.toKind, theme));
    const fromRgba = adjacent ? highlightRgba(fromColor, alpha) : targetRgba(fromColor, alpha);
    const toRgba = adjacent ? highlightRgba(toColor, alpha) : targetRgba(toColor, alpha);
    const vertex = index * VERTICES_PER_EDGE;
    attribute.setXYZW(vertex, fromRgba[0] ?? 0, fromRgba[1] ?? 0, fromRgba[2] ?? 0, fromRgba[3] ?? 0);
    attribute.setXYZW(vertex + 1, toRgba[0] ?? 0, toRgba[1] ?? 0, toRgba[2] ?? 0, toRgba[3] ?? 0);
  }
  attribute.needsUpdate = true;
}

/** Drives the shader's mix between resting and focus-target edge colors. */
export function setEdgeFocusBlend(layer: EdgeLayer, blend: number): void {
  const uniform = layer.object.material.uniforms.uFocusBlend;
  if (uniform === undefined) {
    throw new Error("Edge material is missing the uFocusBlend uniform");
  }
  uniform.value = blend;
}
