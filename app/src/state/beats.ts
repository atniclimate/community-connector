import type { ErrorEnvelopeDto, PresentBeat, PresentCamera, PresentMeasure } from "./state";

/**
 * The presenter beat sheet reader (`public/beats.atni.json`). Beside
 * `snapshot.ts`: the runtime JSON is validated into `PresentBeat[]` before
 * it reaches the store, and a malformed sheet is rejected loudly through
 * the same error path a failed fetch takes (I3, I7-adjacent) instead of
 * being cast and trusted.
 *
 * D-099 (no person name ever on stage) is enforced here as well as in the
 * render path: a beat whose `labelKinds` names a kind in
 * `STAGE_UNLABELED_KINDS` is rejected outright.
 */

export const PERSON_KIND = "person";

/** Entity kinds whose labels never render on stage, whatever a beat says (D-099). */
export const STAGE_UNLABELED_KINDS: readonly string[] = [PERSON_KIND];

export const PRESENT_MEASURES: readonly PresentMeasure[] = ["betweenness_top_n", "single_tie"];
export const PRESENT_CAMERAS: readonly PresentCamera[] = ["hold", "fit", "fly"];

export class BeatSheetError extends Error {
  public readonly envelope: ErrorEnvelopeDto;

  public constructor(message: string) {
    super(message);
    this.name = "BeatSheetError";
    this.envelope = { code: "BeatSheetError", message };
  }
}

type MutableBeat = { -readonly [K in keyof PresentBeat]: PresentBeat[K] };
type BeatFilter = NonNullable<PresentBeat["filter"]>;
type MutableFilter = { -readonly [K in keyof BeatFilter]: BeatFilter[K] };

function isObject(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function isStringArray(value: unknown): value is readonly string[] {
  return Array.isArray(value) && value.every((item) => typeof item === "string");
}

function optionalStringArray(beatId: string, field: string, value: unknown): readonly string[] | undefined {
  if (value === undefined) {
    return undefined;
  }
  if (!isStringArray(value)) {
    throw new BeatSheetError(`Beat ${beatId}: ${field} must be an array of strings`);
  }
  return value;
}

function validateFilter(beatId: string, raw: unknown): BeatFilter {
  if (!isObject(raw)) {
    throw new BeatSheetError(`Beat ${beatId}: filter must be an object`);
  }
  const filter: MutableFilter = {};
  const kinds = optionalStringArray(beatId, "filter.kinds", raw["kinds"]);
  const edgeKinds = optionalStringArray(beatId, "filter.edgeKinds", raw["edgeKinds"]);
  if (kinds !== undefined) {
    filter.kinds = kinds;
  }
  if (edgeKinds !== undefined) {
    filter.edgeKinds = edgeKinds;
  }
  return filter;
}

function validateBeat(raw: unknown, index: number): PresentBeat {
  if (!isObject(raw)) {
    throw new BeatSheetError(`Beat ${index} is not an object`);
  }
  const id = raw["id"];
  if (typeof id !== "string" || id === "") {
    throw new BeatSheetError(`Beat ${index} has no string id`);
  }
  const label = raw["label"];
  if (typeof label !== "string") {
    throw new BeatSheetError(`Beat ${id}: label must be a string`);
  }
  const beat: MutableBeat = { id, label };
  const focusEntityId = raw["focusEntityId"];
  if (focusEntityId !== undefined) {
    if (typeof focusEntityId !== "string") {
      throw new BeatSheetError(`Beat ${id}: focusEntityId must be a string`);
    }
    beat.focusEntityId = focusEntityId;
  }
  const camera = raw["camera"];
  if (camera !== undefined) {
    if (typeof camera !== "string" || !PRESENT_CAMERAS.includes(camera as PresentCamera)) {
      throw new BeatSheetError(`Beat ${id}: unknown camera ${JSON.stringify(camera)}`);
    }
    beat.camera = camera as PresentCamera;
  }
  const measure = raw["measure"];
  if (measure !== undefined) {
    if (typeof measure !== "string" || !PRESENT_MEASURES.includes(measure as PresentMeasure)) {
      throw new BeatSheetError(`Beat ${id}: unknown measure ${JSON.stringify(measure)}`);
    }
    beat.measure = measure as PresentMeasure;
  }
  const topN = raw["topN"];
  if (topN !== undefined) {
    if (typeof topN !== "number" || !Number.isFinite(topN)) {
      throw new BeatSheetError(`Beat ${id}: topN must be a number`);
    }
    beat.topN = topN;
  }
  if (raw["filter"] !== undefined) {
    beat.filter = validateFilter(id, raw["filter"]);
  }
  const labelKinds = optionalStringArray(id, "labelKinds", raw["labelKinds"]);
  if (labelKinds !== undefined) {
    const forbidden = labelKinds.find((kind) => STAGE_UNLABELED_KINDS.includes(kind));
    if (forbidden !== undefined) {
      throw new BeatSheetError(`Beat ${id}: labelKinds may not include ${forbidden} (D-099: no person name on stage)`);
    }
    beat.labelKinds = labelKinds;
  }
  return beat;
}

/**
 * Validates a parsed beat sheet into `PresentBeat[]`, throwing
 * `BeatSheetError` (with an `ErrorEnvelopeDto`) on the first problem: a
 * non-array; a beat without a string id or label; a duplicate id; an
 * unknown `camera` or `measure`; a non-number `topN`; a non-string-array
 * `filter.kinds`, `filter.edgeKinds`, or `labelKinds`; or a `labelKinds`
 * that names a stage-unlabeled kind (person). Unknown keys are dropped.
 */
export function validateBeats(input: unknown): readonly PresentBeat[] {
  if (!Array.isArray(input)) {
    throw new BeatSheetError("Beat sheet must be a JSON array of beats");
  }
  const beats = input.map((raw, index) => validateBeat(raw, index));
  const seen = new Set<string>();
  for (const beat of beats) {
    if (seen.has(beat.id)) {
      throw new BeatSheetError(`Beat ${beat.id}: duplicate id`);
    }
    seen.add(beat.id);
  }
  return beats;
}
