import type { JsonObject, JsonValue, PresentBeat, ProjectionDto } from "../state/state";
import { STAGE_UNLABELED_KINDS } from "../state/beats";
import { edgeEndpoints } from "./projection";

export type MeasureHighlights = {
  readonly ids: ReadonlySet<string>;
  readonly explanations: readonly string[];
};

/** Entities and edges a non-measure beat lights; everything else dims. */
export type BeatHighlights = {
  readonly ids: ReadonlySet<string>;
  readonly edgeIds: ReadonlySet<string>;
};

/** The camera motion a beat change performs (D-103.3). */
export type CameraMove = "none" | "fit" | "fly";

export type CameraMoveContext = {
  readonly present: boolean;
  readonly focusedId: string | null;
  readonly focusChanged: boolean;
  readonly beatChanged: boolean;
  readonly rebuilt: boolean;
};

type RankedMeasure = {
  readonly id: string;
  readonly value: number;
  readonly explanation: string;
};

const EMPTY_IDS: ReadonlySet<string> = new Set<string>();

function objectValue(value: JsonValue | undefined): JsonObject | null {
  return typeof value === "object" && value !== null && !Array.isArray(value)
    ? value as JsonObject
    : null;
}

function measureRecords(response: JsonObject, key: "betweenness" | "degree"): readonly RankedMeasure[] {
  const measures = objectValue(response[key]);
  if (measures === null) {
    throw new Error(`Graph measures response is missing ${key}`);
  }
  return Object.entries(measures).flatMap(([id, value]) => {
    const record = objectValue(value);
    if (record === null || typeof record["explanation"] !== "string") {
      throw new Error(`Graph measures response has invalid ${key} for ${id}`);
    }
    const score = key === "betweenness" ? record["value"] : record["single_tie"];
    if (key === "betweenness" && typeof score !== "number") {
      throw new Error(`Graph measures response has invalid betweenness for ${id}`);
    }
    if (key === "degree" && typeof score !== "boolean") {
      throw new Error(`Graph measures response has invalid degree for ${id}`);
    }
    if (key === "degree" && !score) {
      return [];
    }
    return [{ id, value: typeof score === "number" ? score : 0, explanation: record["explanation"] }];
  });
}

export function measureHighlights(response: JsonObject, beat: PresentBeat): MeasureHighlights {
  const records = beat.measure === undefined
    ? []
    : beat.measure === "betweenness_top_n"
      ? [...measureRecords(response, "betweenness")]
          .sort((left, right) => right.value - left.value || left.id.localeCompare(right.id))
          .slice(0, Math.max(0, Math.floor(beat.topN ?? 0)))
      : [...measureRecords(response, "degree")]
          .sort((left, right) => left.id.localeCompare(right.id));
  return {
    ids: new Set(records.map((record) => record.id)),
    explanations: records.map((record) => record.explanation),
  };
}

/**
 * A kind-filter beat foregrounds its kinds: they take the highlight role and
 * everything else dims, while staying in place. Beats with a measure get
 * their highlight set from the measure call instead (null here).
 */
export function kindBeatHighlights(
  entities: readonly { readonly id: string; readonly kind?: string | null }[],
  beat: PresentBeat | undefined,
): ReadonlySet<string> | null {
  if (beat?.measure !== undefined) {
    return null;
  }
  const kinds = beat?.filter?.kinds;
  if (kinds === undefined || kinds.length === 0) {
    return new Set<string>();
  }
  return new Set(entities.filter((entity) => kinds.includes(entity.kind ?? "")).map((entity) => entity.id));
}

/**
 * An edge-kind beat lights every entity incident to an edge whose kind is in
 * `filter.edgeKinds`, and those edges themselves. Null for measure beats
 * (the measure owns the highlight set) and empty sets when the beat has no
 * edgeKinds. Edges are read from the permission-filtered projection, so a
 * kind the viewer cannot see simply lights nothing (I2).
 */
export function edgeKindBeatHighlights(
  edges: ProjectionDto["edges"],
  beat: PresentBeat | undefined,
): BeatHighlights | null {
  if (beat?.measure !== undefined) {
    return null;
  }
  const edgeKinds = beat?.filter?.edgeKinds;
  if (edgeKinds === undefined || edgeKinds.length === 0) {
    return { ids: new Set<string>(), edgeIds: new Set<string>() };
  }
  const ids = new Set<string>();
  const edgeIds = new Set<string>();
  for (const edge of edges ?? []) {
    const endpoints = edgeEndpoints(edge);
    if (endpoints === null || typeof edge["kind"] !== "string" || !edgeKinds.includes(edge["kind"])) {
      continue;
    }
    edgeIds.add(edge.id);
    ids.add(endpoints.from);
    ids.add(endpoints.to);
  }
  return { ids, edgeIds };
}

/**
 * The highlight set a non-measure beat derives from its filter, or null when
 * a measure owns it. Combination rule when both `kinds` and `edgeKinds` are
 * set: the entity set is the INTERSECTION (entities of those kinds that are
 * incident to an edge of those edge kinds) and only edges whose endpoints
 * both survive stay bright. With only `edgeKinds` every incident entity is
 * lit; with only `kinds` no edge is singled out (today's behavior). A beat's
 * focusEntityId neighborhood is added later by computeFocusSet, not here.
 */
export function beatHighlights(
  projection: ProjectionDto | null,
  beat: PresentBeat | undefined,
): BeatHighlights | null {
  const entities = projection?.entities ?? [];
  const byKind = kindBeatHighlights(entities, beat);
  const byEdgeKind = edgeKindBeatHighlights(projection?.edges, beat);
  if (byKind === null || byEdgeKind === null) {
    return null;
  }
  const hasKinds = (beat?.filter?.kinds?.length ?? 0) > 0;
  const hasEdgeKinds = (beat?.filter?.edgeKinds?.length ?? 0) > 0;
  if (!hasEdgeKinds) {
    return { ids: byKind, edgeIds: EMPTY_IDS };
  }
  if (!hasKinds) {
    return byEdgeKind;
  }
  const ids = new Set([...byEdgeKind.ids].filter((id) => byKind.has(id)));
  const edgeIds = new Set<string>();
  for (const edge of projection?.edges ?? []) {
    const endpoints = edgeEndpoints(edge);
    if (endpoints !== null && byEdgeKind.edgeIds.has(edge.id) && ids.has(endpoints.from) && ids.has(endpoints.to)) {
      edgeIds.add(edge.id);
    }
  }
  return { ids, edgeIds };
}

/**
 * Decides the camera motion for a beat change. Pure so the rule is testable
 * without a renderer; reduced motion is applied by the camera rig (snap),
 * never here (I9).
 *
 * - undefined camera: today's behavior - fly when the focused entity just
 *   changed to something, otherwise fit the beat's positions on a beat
 *   change or rebuild.
 * - "hold": never moves, even with a focusEntityId (opacity-only spotlight).
 * - "fit": always fits on a beat change or rebuild; never flies.
 * - "fly": flies to the focused entity whenever one is set, even if it did
 *   not change; fits when the beat has no focused entity.
 * Outside present mode the beat is ignored and the default rule applies.
 */
export function beatCameraMove(beat: PresentBeat | undefined, context: CameraMoveContext): CameraMove {
  const mode = context.present ? beat?.camera : undefined;
  const beatMoved = context.present && (context.beatChanged || context.rebuilt);
  if (mode === "hold") {
    return "none";
  }
  if (mode === "fit") {
    return beatMoved ? "fit" : "none";
  }
  if (mode === "fly") {
    if (context.focusedId !== null && (context.focusChanged || beatMoved)) {
      return "fly";
    }
    return beatMoved ? "fit" : "none";
  }
  if (context.focusChanged && context.focusedId !== null) {
    return "fly";
  }
  return beatMoved ? "fit" : "none";
}

/**
 * Presenter hotkeys (CS-06) mapped to the beat id each jumps to. `1` and
 * `End` read as digit/named keys; the rest are letters compared
 * case-insensitively. This is the single source of truth for the mapping -
 * the on-screen rail (viz/index.ts) reuses it by key so a button and its
 * hotkey can never drift apart.
 */
const PRESENTER_KEY_BEAT_IDS: Readonly<Record<string, string>> = {
  c: "committees",
  o: "organizations",
  m: "members",
  p: "shared-priorities",
  "1": "one-node",
  end: "constellation",
};

/**
 * Resolves a presenter hotkey (case-insensitive) to the index of the beat
 * whose `id` matches, looked up in `beats` at keypress time - never by
 * position, so reordering `beats.atni.json` cannot make a key jump to the
 * wrong beat. Returns null for a key with no mapping, and null (not a
 * fallback index) when the mapped id is absent from `beats`: the key is
 * simply inert until that beat exists.
 */
export function beatIndexForKey(key: string, beats: readonly PresentBeat[]): number | null {
  const targetId = PRESENTER_KEY_BEAT_IDS[key.toLowerCase()];
  if (targetId === undefined) {
    return null;
  }
  const index = beats.findIndex((beat) => beat.id === targetId);
  return index === -1 ? null : index;
}

/**
 * The stage caption. Measure beats show a count only - "label - N
 * highlighted" - never the per-entity explanations (D-099: counts, never
 * names; D-103.5). `explanations` is null while the measure call is still
 * pending (the bare label shows, never a false zero); once it has returned,
 * the count renders even when it is 0. `highlightCount` defaults to the
 * number of explanations, which is one per highlighted entity.
 */
export function presenterBeatText(
  beat: PresentBeat | undefined,
  explanations: readonly string[] | null,
  highlightCount: number = explanations?.length ?? 0,
): string {
  if (beat === undefined) {
    return "";
  }
  if (beat.measure === undefined || explanations === null) {
    return beat.label;
  }
  return `${beat.label} - ${highlightCount} highlighted`;
}

/**
 * The stage name gate (D-099: no person name ever appears on stage). Takes
 * the label emphasis set the focus pipeline produced - the focused entity
 * plus its neighbors and the beat's highlights, or null when nothing is lit
 * and the normal zoom policy labels the nearest nodes - and restricts it to
 * the beat's `labelKinds`. Outside present mode the emphasis passes through
 * unchanged. In present mode the gate FAILS CLOSED: a missing beat or a
 * beat without `labelKinds` labels nothing (never "everything"), and the
 * kinds in `STAGE_UNLABELED_KINDS` (person) are dropped from `labelKinds`
 * even if a beat lists them, so a person is never labeled on stage whether
 * lit, focused, or context. With `labelKinds`, the result is the emphasis
 * set (every entity when there is none) filtered to the permitted kinds; an
 * empty `labelKinds` labels nothing. The label layer's existing
 * `setEmphasis` renders the result, so this adds no render pass (ADR-004).
 */
export function stageLabelIds(
  entities: readonly { readonly id: string; readonly kind?: string | null }[],
  emphasis: ReadonlySet<string> | null,
  beat: PresentBeat | undefined,
  present: boolean,
): ReadonlySet<string> | null {
  if (!present) {
    return emphasis;
  }
  const labelKinds = (beat?.labelKinds ?? []).filter((kind) => !STAGE_UNLABELED_KINDS.includes(kind));
  if (labelKinds.length === 0) {
    return new Set<string>();
  }
  return new Set(
    entities
      .filter((entity) => (emphasis === null || emphasis.has(entity.id)) && labelKinds.includes(entity.kind ?? ""))
      .map((entity) => entity.id),
  );
}

/** The presenter key that shows or hides the operator's button rail. */
export const RAIL_TOGGLE_KEY = "r";

/**
 * The rail toggle, pure so it is testable without a DOM: `r` (either case)
 * flips `shown`; any other key leaves it alone. The rail starts hidden on
 * every entry to present mode (viz/index.ts resets `shown` on exit), so the
 * stage never shows the operator's buttons unless the operator asks.
 */
export function nextRailShown(shown: boolean, key: string): boolean {
  return key.toLowerCase() === RAIL_TOGGLE_KEY ? !shown : shown;
}

/**
 * Whether the rail element carries the `hidden` attribute: hidden outside
 * present mode regardless of `shown`, and hidden in present mode until the
 * operator toggles it. `hidden` removes it from layout and the tab order, so
 * its buttons are never focusable while hidden.
 */
export function railHidden(present: boolean, shown: boolean): boolean {
  return !present || !shown;
}
