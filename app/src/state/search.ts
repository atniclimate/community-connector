import type { ErrorEnvelopeDto, SearchHitDto } from "./state";

/**
 * Validates the cn-api `search` response payload (a JSON array of SearchHit
 * objects, ADR-003 A-B4). Malformed payloads produce a typed error - never a
 * silent drop (I3).
 */
export type ParsedSearchHits =
  | { readonly hits: readonly SearchHitDto[] }
  | { readonly error: ErrorEnvelopeDto };

function malformed(message: string): ParsedSearchHits {
  return {
    error: {
      code: "ProtocolError",
      message: `Search response malformed: ${message}`,
    },
  };
}

function asSearchHit(value: unknown): SearchHitDto | null {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    return null;
  }
  const record = value as Partial<Record<keyof SearchHitDto, unknown>>;
  const { entity, kind, matched_attr: matchedAttr, snippet } = record;
  if (
    typeof entity !== "string" ||
    typeof kind !== "string" ||
    typeof matchedAttr !== "string" ||
    typeof snippet !== "string"
  ) {
    return null;
  }
  return { entity, kind, matched_attr: matchedAttr, snippet };
}

export function parseSearchHits(value: unknown): ParsedSearchHits {
  if (!Array.isArray(value)) {
    return malformed("expected an array of hits");
  }
  const hits: SearchHitDto[] = [];
  for (const [index, item] of value.entries()) {
    const hit = asSearchHit(item);
    if (hit === null) {
      return malformed(`hit at index ${index} is not a SearchHit object`);
    }
    hits.push(hit);
  }
  return { hits };
}
