import type { JsonObject, JsonValue, PresentBeat } from "../state/state";

export type MeasureHighlights = {
  readonly ids: ReadonlySet<string>;
  readonly explanations: readonly string[];
};

type RankedMeasure = {
  readonly id: string;
  readonly value: number;
  readonly explanation: string;
};

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

export function presenterBeatText(
  beat: PresentBeat | undefined,
  explanations: readonly string[],
): string {
  if (beat === undefined) {
    return "";
  }
  return explanations.length === 0
    ? beat.label
    : `${beat.label}: ${explanations.join("; ")}`;
}
