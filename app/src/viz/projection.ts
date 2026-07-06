import type { KindMeta, ProjectionDto, ProjectionEdgeDto, ProjectionEntityDto } from "../state/state";
import type { Theme } from "../theme/tokens";
import { RENDER_COLORS, RENDER_TOKENS } from "./config";

export type GraphEdge = {
  readonly id: string;
  readonly from: string;
  readonly to: string;
  readonly weight: number;
};

export type GraphEntity = ProjectionEntityDto & {
  readonly kind: string;
};

export function projectedEntities(projection: ProjectionDto | null): readonly GraphEntity[] {
  return (projection?.entities ?? []).flatMap((entity) => {
    if (entity.kind === undefined) {
      return [];
    }
    return [{ ...entity, kind: entity.kind }];
  });
}

export function projectedEdges(projection: ProjectionDto | null): readonly GraphEdge[] {
  return (projection?.edges ?? []).flatMap((edge) => {
    const endpoints = edgeEndpoints(edge);
    if (endpoints === null) {
      return [];
    }
    return [{ id: edge.id, from: endpoints.from, to: endpoints.to, weight: edgeWeight(edge) }];
  });
}

export function edgeEndpoints(edge: ProjectionEdgeDto): { readonly from: string; readonly to: string } | null {
  const from = edge.from ?? edge.source;
  const to = edge.to ?? edge.target;
  if (typeof from !== "string" || typeof to !== "string") {
    return null;
  }
  return { from, to };
}

export function edgeWeight(edge: ProjectionEdgeDto): number {
  return typeof edge.weight === "number" ? edge.weight : RENDER_TOKENS.edge.defaultWeight;
}

export function entityLabel(entity: ProjectionEntityDto, meta: Readonly<Record<string, KindMeta>>): string {
  if (typeof entity.label === "string") {
    return entity.label;
  }
  const attributes = entity.attributes;
  const candidates = ["display_name", "title", "common_name", "scientific_name"];
  if (typeof attributes === "object" && attributes !== null && !Array.isArray(attributes)) {
    const record = attributes as Readonly<Record<string, unknown>>;
    for (const candidate of candidates) {
      const value = attributeText(record[candidate]);
      if (value !== null) {
        return value;
      }
    }
  }
  return meta[entity.kind ?? ""]?.label ?? entity.id;
}

export function attributeText(value: unknown): string | null {
  if (typeof value === "string") {
    return value;
  }
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    return null;
  }
  const tagged = value as { readonly type?: unknown; readonly value?: unknown };
  return tagged.type === "text" && typeof tagged.value === "string" ? tagged.value : null;
}

export function degreeByEntityId(
  entities: readonly ProjectionEntityDto[],
  edges: readonly GraphEdge[],
): ReadonlyMap<string, number> {
  const ids = new Set(entities.map((entity) => entity.id));
  const degrees = new Map(entities.map((entity) => [entity.id, 0]));
  for (const edge of edges) {
    if (ids.has(edge.from)) {
      degrees.set(edge.from, (degrees.get(edge.from) ?? 0) + 1);
    }
    if (ids.has(edge.to)) {
      degrees.set(edge.to, (degrees.get(edge.to) ?? 0) + 1);
    }
  }
  return degrees;
}

export function kindColor(kind: string, theme: Theme | null): string {
  return theme?.tokens[`kind.${kind}.base`]?.hex ?? RENDER_COLORS.fallbackKind;
}
