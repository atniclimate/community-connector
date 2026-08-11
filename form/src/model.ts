/**
 * Template-driven field model for the remote form (blueprint 4.1, R2). Pure
 * logic: extract the entry form FROM the group template's kinds[].attributes[],
 * convert raw UI inputs into typed payload values, and compute advisory-only
 * validation (the core re-validates authoritatively at staging and approval,
 * I2 - nothing here decides trust).
 *
 * This solves the same problem as app/src/ui/forms/model.ts and mirrors its
 * shape deliberately, but is an independent implementation (the form shares no
 * runtime code with app/, blueprint section 4). Two intentional differences
 * from the in-app model:
 *  - `media` attributes are EXCLUDED entirely (see formModel).
 *  - timestamps are emitted as ISO strings, not epoch numbers - that lives in
 *    envelope.ts, because the remote payload is parsed by the strict Rust
 *    InnerPayload struct where captured_at/consent_affirmed_at are `String`.
 */
import type { JsonObject, JsonValue } from "./json";

/** Advisory caps mirroring the core's authoritative limits (UTF-8 bytes). */
export const TEXT_MAX_BYTES = 2000;
export const TAGS_MAX_ITEMS = 20;

function utf8Bytes(text: string): number {
  return new TextEncoder().encode(text).length;
}

export type AttrType =
  | "text"
  | "number"
  | "enum"
  | "tags"
  | "date"
  | "geo"
  | "link"
  | "media";

export type FormAttr = {
  readonly id: string;
  readonly attrType: Exclude<AttrType, "media">;
  readonly required: boolean;
  /** Enum choices; empty for non-enum types. */
  readonly values: readonly string[];
  /** Shown read-only (pilot entries are T1; per-field tier UX is post-pilot). */
  readonly defaultVisibility: string | null;
};

export type FormKind = {
  readonly id: string;
  readonly label: string;
  readonly attributes: readonly FormAttr[];
};

export type FormModel = {
  readonly kinds: readonly FormKind[];
};

function asString(value: JsonValue | undefined): string | null {
  return typeof value === "string" ? value : null;
}

/**
 * Extracts the form model from a parsed group-template JSON object.
 *
 * `media`-typed attributes are dropped here so they are neither rendered nor
 * assembled into the payload. This is architecturally forced, not arbitrary:
 * ADR-005 D6 caps the outer envelope at single-digit KB
 * (DEFAULT_MAX_ENVELOPE_BYTES = 8192 in cn-ingest/src/envelope.rs), and a photo
 * would exceed that by orders of magnitude. Blueprint 4.1's R2-type list for
 * the form omits `media` for exactly this reason. Both fixture templates carry
 * a media attribute (fisheries-committee `family_canoe_photo` and `flyer`), so
 * this path is exercised on real fixtures, not a hypothetical.
 */
export function formModel(template: JsonObject): FormModel {
  const kinds: FormKind[] = [];
  const rawKinds = Array.isArray(template["kinds"]) ? template["kinds"] : [];
  for (const rawKind of rawKinds) {
    if (typeof rawKind !== "object" || rawKind === null || Array.isArray(rawKind)) {
      continue;
    }
    const kind = rawKind as JsonObject;
    const id = asString(kind["id"]);
    if (id === null) {
      continue;
    }
    const attributes: FormAttr[] = [];
    const rawAttrs = Array.isArray(kind["attributes"]) ? kind["attributes"] : [];
    for (const rawAttr of rawAttrs) {
      if (typeof rawAttr !== "object" || rawAttr === null || Array.isArray(rawAttr)) {
        continue;
      }
      const attr = rawAttr as JsonObject;
      const attrId = asString(attr["id"]);
      const attrType = asString(attr["type"]);
      if (attrId === null || attrType === null) {
        continue;
      }
      // media excluded: too large for the D6 envelope cap; see the doc comment.
      if (attrType === "media") {
        continue;
      }
      attributes.push({
        id: attrId,
        attrType: attrType as Exclude<AttrType, "media">,
        required: attr["required"] === true,
        values: Array.isArray(attr["values"])
          ? attr["values"].filter((v): v is string => typeof v === "string")
          : [],
        defaultVisibility: asString(attr["default_visibility"]),
      });
    }
    kinds.push({ id, label: asString(kind["label"]) ?? id, attributes });
  }
  return { kinds };
}

/** One field's raw UI input: a string, or a string list for tags. */
export type RawFieldValue = string | readonly string[];

/**
 * Converts a raw UI input into the typed payload value for its attribute, or
 * undefined when the input is empty (empty optional fields are omitted from the
 * payload entirely). Geo shape matches the in-app precedent: {lat, lon} for a
 * "lat, lon" point, otherwise {name}.
 */
export function fieldValue(attr: FormAttr, raw: RawFieldValue): JsonValue | undefined {
  if (Array.isArray(raw)) {
    const items = raw.map((item) => item.trim()).filter((item) => item.length > 0);
    return items.length > 0 ? items : undefined;
  }
  const text = (raw as string).trim();
  if (text.length === 0) {
    return undefined;
  }
  switch (attr.attrType) {
    case "number": {
      const parsed = Number(text);
      return Number.isFinite(parsed) ? parsed : undefined;
    }
    case "tags":
      return text
        .split(/[\n,]/)
        .map((item) => item.trim())
        .filter((item) => item.length > 0);
    case "geo": {
      const parts = text.split(",").map((part) => part.trim());
      if (parts.length === 2) {
        const lat = Number(parts[0]);
        const lon = Number(parts[1]);
        if (Number.isFinite(lat) && Number.isFinite(lon)) {
          return { lat, lon };
        }
      }
      return { name: text };
    }
    default:
      return text;
  }
}

/**
 * Advisory issues for one field (empty list = no advisory finding). The core's
 * validation report is the authoritative one (I2).
 */
export function advisoryIssues(attr: FormAttr, raw: RawFieldValue): readonly string[] {
  const issues: string[] = [];
  const value = fieldValue(attr, raw);
  if (value === undefined) {
    if (attr.required) {
      issues.push("This field is required.");
    }
    if (attr.attrType === "number" && typeof raw === "string" && raw.trim().length > 0) {
      issues.push("Enter a number.");
    }
    return issues;
  }
  if (typeof value === "string" && utf8Bytes(value) > TEXT_MAX_BYTES) {
    issues.push(`Keep this under ${TEXT_MAX_BYTES} bytes.`);
  }
  if (Array.isArray(value)) {
    if (value.length > TAGS_MAX_ITEMS) {
      issues.push(`List at most ${TAGS_MAX_ITEMS} items.`);
    }
    if (value.some((item) => typeof item === "string" && utf8Bytes(item) > TEXT_MAX_BYTES)) {
      issues.push(`Keep each item under ${TEXT_MAX_BYTES} bytes.`);
    }
  }
  if (
    typeof value === "object" &&
    value !== null &&
    !Array.isArray(value) &&
    typeof (value as { readonly name?: unknown }).name === "string" &&
    utf8Bytes((value as { readonly name: string }).name) > TEXT_MAX_BYTES
  ) {
    issues.push(`Keep the place name under ${TEXT_MAX_BYTES} bytes.`);
  }
  if (
    attr.attrType === "enum" &&
    typeof value === "string" &&
    attr.values.length > 0 &&
    !attr.values.includes(value)
  ) {
    issues.push("Pick one of the listed choices.");
  }
  return issues;
}

/**
 * Assembles the typed fields map for the payload (empty optional fields
 * omitted). media attributes never appear here because formModel already
 * excluded them.
 */
export function buildFields(
  attributes: readonly FormAttr[],
  raw: Readonly<Record<string, RawFieldValue>>,
): Record<string, JsonValue> {
  const fields: Record<string, JsonValue> = {};
  for (const attr of attributes) {
    const rawValue = raw[attr.id];
    if (rawValue === undefined) {
      continue;
    }
    const value = fieldValue(attr, rawValue);
    if (value !== undefined) {
      fields[attr.id] = value;
    }
  }
  return fields;
}

/**
 * The structural consent gate (D-030): with the checkbox unchecked nothing
 * sends, regardless of field state. Advisory field issues also block submit so
 * obviously-broken payloads are caught before sealing; the core still
 * re-validates authoritatively after decryption.
 */
export function canSubmit(
  attributes: readonly FormAttr[],
  fields: Readonly<Record<string, RawFieldValue>>,
  consentAffirmed: boolean,
): boolean {
  if (!consentAffirmed) {
    return false;
  }
  return attributes.every((attr) => advisoryIssues(attr, fields[attr.id] ?? "").length === 0);
}
