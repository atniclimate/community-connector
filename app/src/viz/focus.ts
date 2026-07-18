import { Color, Matrix4, Object3D } from "three";
import type { ProjectionDto } from "../state/state";
import { scaleChroma, shiftLightness } from "../theme/color";
import type { Theme } from "../theme/tokens";
import { RENDER_TOKENS } from "./config";
import { degreeToScale, type NodeLayer } from "./nodes";
import { kindColor, projectedEdges } from "./projection";

// Focus-mode dim/highlight (P1.2). The focus SET is derived here from the
// permission-filtered projection the store already holds; no permission logic
// lives in this module (I2). Focus state itself comes from the app state
// machine (view.mode / view.focusedEntityId).

export type FocusSet = {
  readonly focusedId: string;
  readonly neighborIds: ReadonlySet<string>;
  readonly adjacentEdgeIds: ReadonlySet<string>;
};

export type NodeFocusRole = "base" | "focused" | "neighbor" | "unrelated";

const ZERO = 0;
const UNIT = 1;
const MATRIX = new Matrix4();
const DUMMY = new Object3D();
const COLOR = new Color();

// Fallback variant math mirrors deriveKindTokens in app/src/theme/derive.ts so
// focus colors match the theme pipeline's hover/dim tokens when a template
// omits them.
const HOVER_LIGHTNESS_DELTA = 0.08;
const DIM_CHROMA_SCALE = 0.35;

export function computeFocusSet(projection: ProjectionDto, focusedId: string | null): FocusSet | null {
  if (focusedId === null) {
    return null;
  }
  const neighborIds = new Set<string>();
  const adjacentEdgeIds = new Set<string>();
  for (const edge of projectedEdges(projection)) {
    if (edge.from !== focusedId && edge.to !== focusedId) {
      continue;
    }
    adjacentEdgeIds.add(edge.id);
    const other: string = edge.from === focusedId ? edge.to : edge.from;
    if (other !== focusedId) {
      neighborIds.add(other);
    }
  }
  return { focusedId, neighborIds, adjacentEdgeIds };
}

export function focusRole(entityId: string, focus: FocusSet | null): NodeFocusRole {
  if (focus === null) {
    return "base";
  }
  if (entityId === focus.focusedId) {
    return "focused";
  }
  return focus.neighborIds.has(entityId) ? "neighbor" : "unrelated";
}

export function nodeFocusHex(kind: string, role: NodeFocusRole, theme: Theme | null): string {
  const base = kindColor(kind, theme);
  if (role === "focused") {
    return theme?.tokens[`kind.${kind}.hover`]?.hex ?? shiftLightness(base, HOVER_LIGHTNESS_DELTA);
  }
  if (role === "unrelated") {
    return theme?.tokens[`kind.${kind}.dim`]?.hex ?? scaleChroma(base, DIM_CHROMA_SCALE);
  }
  return base;
}

export function nodeFocusScaleFactor(role: NodeFocusRole): number {
  if (role === "focused") {
    return RENDER_TOKENS.node.selectedScale;
  }
  if (role === "unrelated") {
    return RENDER_TOKENS.node.dimmedScale;
  }
  return UNIT;
}

/**
 * Rewrites node instance colors and scales for the given focus set. Scale is
 * recomputed from the entity degree (absolute, not multiplied into the current
 * matrix), so repeated application never compounds.
 */
export function applyFocusToNodeLayer(
  layer: NodeLayer,
  projection: ProjectionDto,
  theme: Theme | null,
  degrees: ReadonlyMap<string, number>,
  focus: FocusSet | null,
): void {
  const kinds = new Map((projection.entities ?? []).map((entity) => [entity.id, entity.kind ?? ""]));
  for (const record of layer.records) {
    for (const [instanceId, entityId] of record.entityIds.entries()) {
      const role = focusRole(entityId, focus);
      COLOR.set(nodeFocusHex(kinds.get(entityId) ?? "", role, theme));
      record.mesh.setColorAt(instanceId, COLOR);
      record.mesh.getMatrixAt(instanceId, MATRIX);
      MATRIX.decompose(DUMMY.position, DUMMY.quaternion, DUMMY.scale);
      const baseScale = degreeToScale(degrees.get(entityId) ?? RENDER_TOKENS.node.minDegree);
      DUMMY.scale.setScalar(Math.max(RENDER_TOKENS.node.minRadius, baseScale * nodeFocusScaleFactor(role)));
      DUMMY.updateMatrix();
      record.mesh.setMatrixAt(instanceId, DUMMY.matrix);
    }
    record.mesh.instanceMatrix.needsUpdate = true;
    if (record.mesh.instanceColor !== null) {
      record.mesh.instanceColor.needsUpdate = true;
    }
  }
}

/**
 * Animates the 0..1 focus blend consumed by the edge shader's dual color
 * buffers and the halo dim. Under reduced motion the value snaps to its
 * target in a single update, so the end state is identical and only the
 * animation is suppressed (I9).
 */
export class FocusBlend {
  private current = ZERO;
  private target = ZERO;

  public get value(): number {
    return this.current;
  }

  public setTarget(target: number): void {
    this.target = Math.min(UNIT, Math.max(ZERO, target));
  }

  public update(elapsedMs: number, reducedMotion: boolean): boolean {
    if (this.current === this.target) {
      return false;
    }
    if (reducedMotion) {
      this.current = this.target;
      return true;
    }
    const step = elapsedMs / RENDER_TOKENS.focus.blendMs;
    this.current = this.current < this.target
      ? Math.min(this.target, this.current + step)
      : Math.max(this.target, this.current - step);
    return true;
  }
}
