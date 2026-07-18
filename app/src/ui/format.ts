import type { EntityDetailDto, JsonValue } from "../state/state";

/**
 * Pure formatting helpers for the detail panel (P1.5). Everything here renders
 * ONLY what the cn-api `entity_detail` payload hands the app - there are no
 * viewer-role checks in this module or anywhere else in the app layer (I2).
 * Provenance depth (one line for members, full custody chain for governance,
 * D-033) is decided by the core: the panel shows a chain exactly when the
 * payload carries one.
 */

/** Literal marker required on user-visible tier prose (D-023, D-044.4). */
export const DRAFT_REVIEW_MARKER = "DRAFT - PENDING HUMAN REVIEW (D-023)";

export const TIER_CODES = ["T0", "T1", "T2", "T3"] as const;
export type TierCode = (typeof TIER_CODES)[number];

/**
 * Generic plain-language secondaries for the TSDF tier codes (D-032: codes are
 * the primary UI language; these appear secondarily). Generic wording only -
 * no community-specific vocabulary (G1). Rendered together with
 * DRAFT_REVIEW_MARKER.
 */
export const TIER_PLAIN: Readonly<Record<TierCode, string>> = {
  T0: "May be shared publicly",
  T1: "Shared across the trusted network",
  T2: "Shared within this group only",
  T3: "Visible only to the record owner",
};

export function isTierCode(value: unknown): value is TierCode {
  return typeof value === "string" && (TIER_CODES as readonly string[]).includes(value);
}

export function humanizeAttrId(attrId: string): string {
  return attrId.replaceAll(/[_-]+/g, " ").trim();
}

function isJsonObject(value: unknown): value is Readonly<Record<string, JsonValue>> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function formatGeo(value: JsonValue): string | null {
  if (!isJsonObject(value)) {
    return null;
  }
  const point = value.point;
  if (isJsonObject(point) && typeof point.lat === "number" && typeof point.lon === "number") {
    return `${point.lat}, ${point.lon}`;
  }
  const region = value.region;
  if (isJsonObject(region) && typeof region.name === "string") {
    return region.name;
  }
  return null;
}

function formatLink(value: JsonValue): string | null {
  if (isJsonObject(value) && typeof value.url === "string") {
    return value.url;
  }
  return null;
}

function formatTags(value: JsonValue): string | null {
  if (Array.isArray(value) && value.every((tag) => typeof tag === "string")) {
    return value.join(", ");
  }
  return null;
}

/**
 * Formats a tagged cn-model AttributeValue ({"type": ..., "value": ...}) for
 * reading. Unknown shapes fall back to raw JSON so nothing is silently
 * dropped (I3).
 */
export function formatAttributeValue(value: JsonValue | undefined): string {
  if (value === undefined || value === null) {
    return "";
  }
  if (typeof value === "string" || typeof value === "number" || typeof value === "boolean") {
    return String(value);
  }
  if (isJsonObject(value) && typeof value.type === "string") {
    const inner = value.value;
    switch (value.type) {
      case "text":
      case "enum":
      case "date":
      case "media":
        if (typeof inner === "string") {
          return inner;
        }
        break;
      case "number":
        if (typeof inner === "number") {
          return String(inner);
        }
        break;
      case "tags": {
        const tags = inner === undefined ? null : formatTags(inner);
        if (tags !== null) {
          return tags;
        }
        break;
      }
      case "geo": {
        const geo = inner === undefined ? null : formatGeo(inner);
        if (geo !== null) {
          return geo;
        }
        break;
      }
      case "link": {
        const link = inner === undefined ? null : formatLink(inner);
        if (link !== null) {
          return link;
        }
        break;
      }
      default:
        break;
    }
  }
  return JSON.stringify(value);
}

export type DetailAttributeView = {
  readonly id: string;
  readonly label: string;
  readonly valueText: string;
  readonly visibility: string | null;
  readonly tier: TierCode | null;
};

/**
 * Flattens `entity_detail` attributes ({attr: {value, visibility?, tier?}})
 * into render rows. Visibility and tier appear exactly when the payload
 * carries them (own-record settings today; whatever the core adds tomorrow).
 */
export function detailAttributeViews(detail: EntityDetailDto): readonly DetailAttributeView[] {
  const attributes = detail.attributes;
  if (!isJsonObject(attributes)) {
    return [];
  }
  return Object.entries(attributes).map(([id, raw]) => {
    if (!isJsonObject(raw)) {
      return {
        id,
        label: humanizeAttrId(id),
        valueText: formatAttributeValue(raw),
        visibility: null,
        tier: null,
      };
    }
    return {
      id,
      label: humanizeAttrId(id),
      valueText: formatAttributeValue(raw.value),
      visibility: typeof raw.visibility === "string" ? raw.visibility : null,
      tier: isTierCode(raw.tier) ? raw.tier : null,
    };
  });
}

export type ProvenanceView = {
  readonly line: string;
  readonly chain: readonly string[];
};

export type FormatFinding = {
  readonly code: "MalformedProvenance";
  readonly message: string;
};

export type ProvenanceResult = {
  readonly view: ProvenanceView | null;
  readonly finding: FormatFinding | null;
};

function dateText(value: JsonValue | undefined): string {
  if (typeof value === "number" && Number.isFinite(value)) {
    return new Date(value).toISOString().slice(0, 10);
  }
  if (typeof value === "string") {
    return value;
  }
  return "unknown date";
}

function actorText(value: JsonValue | undefined): string {
  if (typeof value === "string") {
    return value;
  }
  if (isJsonObject(value)) {
    if (typeof value.human === "string") {
      return `person ${value.human}`;
    }
    if (isJsonObject(value.agent) && typeof value.agent.agent_id === "string") {
      return `agent ${value.agent.agent_id}`;
    }
  }
  return "unknown actor";
}

function custodyChain(
  value: JsonValue | undefined,
): { readonly chain: readonly string[]; readonly finding: FormatFinding | null } {
  if (value === undefined) {
    return { chain: [], finding: null };
  }
  if (!Array.isArray(value)) {
    return {
      chain: [],
      finding: {
        code: "MalformedProvenance",
        message: "Entity detail provenance custody must be an array",
      },
    };
  }
  const chain: string[] = [];
  for (const [index, event] of value.entries()) {
    if (
      !isJsonObject(event) ||
      typeof event.action !== "string" ||
      (typeof event.at !== "number" && typeof event.at !== "string")
    ) {
      return {
        chain: [],
        finding: {
          code: "MalformedProvenance",
          message: `Entity detail provenance custody event ${index} is malformed`,
        },
      };
    }
    const note = typeof event.note === "string" ? ` - ${event.note}` : "";
    chain.push(`${event.action} by ${actorText(event.actor)} (${dateText(event.at)})${note}`);
  }
  return { chain, finding: null };
}

/**
 * Builds the provenance section from the payload's `provenance` field when it
 * exists. Returns null when the payload carries no provenance - the section is
 * omitted, never inferred (I2).
 */
export function extractProvenance(detail: EntityDetailDto): ProvenanceResult {
  const raw = detail.provenance;
  if (raw === undefined || raw === null) {
    return { view: null, finding: null };
  }
  if (!isJsonObject(raw) || typeof raw.line !== "string") {
    return {
      view: null,
      finding: {
        code: "MalformedProvenance",
        message: "Entity detail provenance must contain a one-line summary",
      },
    };
  }
  const custody = custodyChain(raw.custody);
  return {
    view: { line: raw.line, chain: custody.chain },
    finding: custody.finding,
  };
}
