import { Group, Quaternion, Vector3, type Camera } from "three";
import { Text } from "troika-three-text";
import labelFontWoff from "@fontsource/atkinson-hyperlegible/files/atkinson-hyperlegible-latin-400-normal.woff";
import type { KindMeta, ProjectionDto } from "../state/state";
import type { Theme } from "../theme/tokens";
import { RENDER_COLORS, RENDER_TOKENS } from "./config";
import type { LayoutResult } from "./layout";
import { degreeToScale } from "./nodes";
import { entityLabel } from "./projection";
import type { QualityTier } from "./quality";

// Labels are deliberately static: they appear and disappear instantly with no
// fade or slide, so the reduced-motion contract holds in both media states.
// Billboarding is a fixed orientation toward the camera, not an animation.

export type LabelCandidate = {
  readonly id: string;
  readonly text: string;
  readonly position: Vector3;
  readonly degree: number;
};

export type LabelLayer = {
  readonly group: Group;
  readonly update: (camera: Camera, elapsedMs: number) => boolean;
  readonly dispose: () => void;
};

export type BuildLabelLayerArgs = {
  readonly projection: ProjectionDto;
  readonly layout: LayoutResult;
  readonly kindMeta: Readonly<Record<string, KindMeta>>;
  readonly theme: Theme | null;
  readonly degrees: ReadonlyMap<string, number>;
  readonly tier: QualityTier;
  readonly onNeedsRender: () => void;
};

const ZERO = 0;
const UNIT = 1;
const ELLIPSIS = "...";
const NOOP_RAYCAST = (): void => {
  // Labels are read-only; picking raycasts node meshes exclusively (picking.ts),
  // and this noop keeps any future generic raycast from hitting label glyphs.
};

export function truncateLabel(text: string): string {
  const limit = RENDER_TOKENS.label.maxChars;
  if (text.length <= limit) {
    return text;
  }
  return `${text.slice(ZERO, limit - ELLIPSIS.length).trimEnd()}${ELLIPSIS}`;
}

export function visibleLabelCap(tier: QualityTier): number {
  switch (tier) {
    case "A":
      return RENDER_TOKENS.label.capTierA;
    case "B":
      return RENDER_TOKENS.label.capTierB;
    case "C":
      return RENDER_TOKENS.label.capTierC;
    case "D":
      return RENDER_TOKENS.label.capTierD;
    default:
      return RENDER_TOKENS.label.capTierD;
  }
}

export function labelCandidates(
  projection: ProjectionDto,
  layout: LayoutResult,
  kindMeta: Readonly<Record<string, KindMeta>>,
  degrees: ReadonlyMap<string, number>,
): readonly LabelCandidate[] {
  return (projection.entities ?? []).flatMap((entity) => {
    const position = layout.positions.get(entity.id);
    if (position === undefined) {
      return [];
    }
    return [
      {
        id: entity.id,
        text: truncateLabel(entityLabel(entity, kindMeta)),
        position,
        degree: degrees.get(entity.id) ?? RENDER_TOKENS.node.minDegree,
      },
    ];
  });
}

function hubShare(degree: number): number {
  const clamped = Math.min(RENDER_TOKENS.node.referenceDegree, Math.max(ZERO, degree));
  return clamped / RENDER_TOKENS.node.referenceDegree;
}

// Zoom-adaptive policy: a label is eligible when its hub-adjusted distance to the
// camera is inside the visibility radius, so zooming toward a cluster reveals its
// labels while high-degree hubs stay labeled from farther out. Density culling:
// eligible labels are ranked by adjusted distance and hard-capped per quality tier,
// so the layer never renders every label at pilot scale.
export function selectVisibleLabels(
  candidates: readonly LabelCandidate[],
  cameraPosition: Vector3,
  tier: QualityTier,
): readonly LabelCandidate[] {
  const cap = visibleLabelCap(tier);
  if (cap <= ZERO) {
    return [];
  }
  const scored = candidates.flatMap((candidate) => {
    const adjusted =
      candidate.position.distanceTo(cameraPosition) /
      (UNIT + RENDER_TOKENS.label.hubBoost * hubShare(candidate.degree));
    if (adjusted > RENDER_TOKENS.label.visibleDistance) {
      return [];
    }
    return [{ candidate, adjusted }];
  });
  scored.sort(
    (left, right) => left.adjusted - right.adjusted || left.candidate.id.localeCompare(right.candidate.id),
  );
  return scored.slice(ZERO, cap).map((entry) => entry.candidate);
}

function themeHex(theme: Theme | null, key: string, fallback: string): string {
  return theme?.tokens[key]?.hex ?? fallback;
}

function makeLabel(theme: Theme | null): Text {
  const label = new Text();
  label.font = labelFontWoff;
  label.fontSize = RENDER_TOKENS.label.fontSize;
  label.color = themeHex(theme, "label.text", RENDER_COLORS.labelText);
  label.outlineColor = themeHex(theme, "label.outline", RENDER_COLORS.labelOutline);
  label.outlineWidth = RENDER_TOKENS.label.fontSize * RENDER_TOKENS.label.outlineWidthRatio;
  label.anchorX = "center";
  label.anchorY = "bottom";
  label.material.depthTest = false;
  label.material.depthWrite = false;
  label.renderOrder = RENDER_TOKENS.label.renderOrder;
  label.visible = false;
  label.raycast = NOOP_RAYCAST;
  return label;
}

type LabelSlot = {
  readonly mesh: Text;
  entityId: string | null;
};

function anchorFor(candidate: LabelCandidate): Vector3 {
  const radius = degreeToScale(candidate.degree);
  return candidate.position
    .clone()
    .setY(candidate.position.y + radius * RENDER_TOKENS.label.offsetGap + RENDER_TOKENS.label.offsetPad);
}

function assignSlot(slot: LabelSlot, candidate: LabelCandidate, onNeedsRender: () => void): void {
  const textChanged = slot.entityId !== candidate.id;
  slot.entityId = candidate.id;
  slot.mesh.position.copy(anchorFor(candidate));
  slot.mesh.visible = true;
  if (textChanged) {
    slot.mesh.text = candidate.text;
    slot.mesh.sync(onNeedsRender);
  }
}

function applyVisible(
  slots: readonly LabelSlot[],
  visible: readonly LabelCandidate[],
  onNeedsRender: () => void,
): boolean {
  const kept = new Map(slots.flatMap((slot) => (slot.entityId === null ? [] : [[slot.entityId, slot] as const])));
  const incoming = new Set(visible.map((candidate) => candidate.id));
  let changed = false;
  for (const slot of slots) {
    if (slot.entityId !== null && !incoming.has(slot.entityId)) {
      slot.entityId = null;
      slot.mesh.visible = false;
      changed = true;
    }
  }
  const free = slots.filter((slot) => slot.entityId === null);
  for (const candidate of visible) {
    const existing = kept.get(candidate.id);
    if (existing !== undefined && existing.entityId === candidate.id) {
      continue;
    }
    const slot = free.pop();
    if (slot === undefined) {
      break;
    }
    assignSlot(slot, candidate, onNeedsRender);
    changed = true;
  }
  return changed;
}

export function buildLabelLayer(args: BuildLabelLayerArgs): LabelLayer {
  const group = new Group();
  const candidates = labelCandidates(args.projection, args.layout, args.kindMeta, args.degrees);
  const cap = Math.min(visibleLabelCap(args.tier), candidates.length);
  const slots: LabelSlot[] = [];
  for (let index = 0; index < cap; index += UNIT) {
    const slot: LabelSlot = { mesh: makeLabel(args.theme), entityId: null };
    slots.push(slot);
    group.add(slot.mesh);
  }
  const lastCamera = new Vector3(Number.NaN, Number.NaN, Number.NaN);
  const orientation = new Quaternion();
  let sinceRecomputeMs = Number.POSITIVE_INFINITY;
  const update = (camera: Camera, elapsedMs: number): boolean => {
    sinceRecomputeMs += elapsedMs;
    let changed = false;
    const moved = lastCamera.distanceToSquared(camera.position);
    const shouldRecompute =
      Number.isNaN(moved) ||
      (sinceRecomputeMs >= RENDER_TOKENS.label.updateIntervalMs && moved > RENDER_TOKENS.label.cameraEpsilonSq);
    if (shouldRecompute) {
      changed = applyVisible(slots, selectVisibleLabels(candidates, camera.position, args.tier), args.onNeedsRender);
      lastCamera.copy(camera.position);
      sinceRecomputeMs = ZERO;
    }
    camera.getWorldQuaternion(orientation);
    for (const slot of slots) {
      if (slot.mesh.visible) {
        slot.mesh.quaternion.copy(orientation);
      }
    }
    return changed;
  };
  return {
    group,
    update,
    dispose: () => {
      for (const slot of slots) {
        slot.mesh.dispose();
      }
    },
  };
}
