import type {
  ContinuanceDto,
  ErrorEnvelopeDto,
  IntroductionDto,
  NeighborhoodDto,
  PathDto,
  RecordMirrorDto,
  SearchHitDto,
} from "./state";

/**
 * Validates the cn-api responses for routing, neighborhood, continuance,
 * introductions, and the record mirror. Only the fields the app reads are
 * checked; a malformed payload is a typed ProtocolError, never a silent
 * drop (I3).
 */
export type Parsed<T> = { readonly value: T } | { readonly error: ErrorEnvelopeDto };

type Fields = Record<string, unknown>;

function malformed(what: string, message: string): { readonly error: ErrorEnvelopeDto } {
  return { error: { code: "ProtocolError", message: `${what} response malformed: ${message}` } };
}

function isObject(value: unknown): value is Fields {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function has(value: unknown, type: "string" | "number" | "boolean", keys: readonly string[]): value is Fields {
  return isObject(value) && keys.every((key) => typeof value[key] === type);
}

function isStringArray(value: unknown): value is string[] {
  return Array.isArray(value) && value.every((item) => typeof item === "string");
}

function everyValue(value: unknown, check: (item: unknown) => boolean): boolean {
  return isObject(value) && Object.values(value).every(check);
}

export function parsePath(value: unknown): Parsed<PathDto | null> {
  if (value === null) {
    return { value: null };
  }
  if (!has(value, "number", ["cost"]) || !isStringArray(value.nodes) || !isStringArray(value.edges)) {
    return malformed("Path", "expected null or an object with nodes, edges, and cost");
  }
  return { value: { nodes: value.nodes, edges: value.edges, cost: value.cost as number } };
}

export function parseNeighborhood(value: unknown): Parsed<NeighborhoodDto> {
  if (!has(value, "string", ["center"]) || !Array.isArray(value.layers) || !value.layers.every(isStringArray)) {
    return malformed("Neighborhood", "expected a center and layers of entity ids");
  }
  return { value: value as unknown as NeighborhoodDto };
}

export function parseContinuance(value: unknown): Parsed<ContinuanceDto> {
  const point = (item: unknown): boolean =>
    has(item, "string", ["entity", "explanation"]) && has(item, "number", ["parts"]);
  const bridge = (item: unknown): boolean =>
    has(item, "string", ["edge", "from", "to", "explanation"]) && has(item, "number", ["from_side", "to_side"]);
  if (
    !isObject(value) ||
    !everyValue(value.articulation_points, point) ||
    !everyValue(value.bridges, bridge) ||
    !everyValue(value.components, (size) => typeof size === "number")
  ) {
    return malformed("Continuance", "expected articulation_points, bridges, and components maps");
  }
  return { value: value as unknown as ContinuanceDto };
}

export function parseIntroductions(value: unknown): Parsed<readonly IntroductionDto[]> {
  const suggestion = (item: unknown): boolean =>
    has(item, "string", ["candidate", "explanation"]) &&
    has(item, "number", ["score"]) &&
    Array.isArray(item.introducers) &&
    item.introducers.every((introducer) => has(introducer, "string", ["entity"]) && has(introducer, "boolean", ["willing"]));
  if (!Array.isArray(value) || !value.every(suggestion)) {
    return malformed("Introductions", "expected an array of suggestions");
  }
  return { value: value as IntroductionDto[] };
}

function isVisibilityMirror(value: unknown): boolean {
  return has(value, "string", ["visibility", "ceiling", "effective_tier"]) && has(value, "boolean", ["tier_capped"]);
}

export function parseMirror(value: unknown): Parsed<RecordMirrorDto> {
  const entity = (item: unknown): boolean =>
    isObject(item) && isVisibilityMirror(item.presence) && everyValue(item.attributes, isVisibilityMirror);
  if (!isObject(value) || !everyValue(value.entities, entity)) {
    return malformed("Record mirror", "expected entities with presence and attribute mirrors");
  }
  return { value: value as unknown as RecordMirrorDto };
}

/** Groups search hits by entity in first-hit order, leaving out `exclude`. */
export function candidatesFromHits(
  hits: readonly SearchHitDto[],
  exclude: string | null,
): { readonly entity: string; readonly hits: readonly SearchHitDto[] }[] {
  const byEntity = new Map<string, SearchHitDto[]>();
  for (const hit of hits) {
    if (hit.entity === exclude) {
      continue;
    }
    const list = byEntity.get(hit.entity) ?? [];
    list.push(hit);
    byEntity.set(hit.entity, list);
  }
  return [...byEntity].map(([entity, grouped]) => ({ entity, hits: grouped }));
}
