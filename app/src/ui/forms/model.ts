/**
 * P3.6 entry-form model: pure logic for rendering an entry form FROM the
 * group template (R2 - the schema's kinds[].attributes[] consumed) and for
 * assembling the inner-payload-shaped submission object the intake queue
 * stages (docs/blueprints/intake-pipeline.md section 4).
 *
 * Client-side validation here is ADVISORY UX ONLY: the core re-validates
 * at staging and at approval (authoritative, I2). Nothing in this module
 * decides trust.
 */

import type { JsonObject, JsonValue } from "../../state/state";

export const SUBMISSION_VERSION = "0.1.0";

/** Advisory caps MIRRORING the core's authoritative limits
 * (cn-ingest PAYLOAD_TEXT_MAX / TAGS_MAX_ITEMS): one documented limit
 * set, measured in the SAME unit as the core - UTF-8 BYTES, not UTF-16
 * code units (round-3 F9). */
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
  readonly attrType: AttrType;
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

/** Extracts the form model from a parsed group-template JSON object. */
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
      attributes.push({
        id: attrId,
        attrType: attrType as AttrType,
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
 * Converts a raw UI input into the typed payload value for its attribute,
 * or undefined when the input is empty (empty optional fields are omitted
 * from the payload entirely).
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
      // The core's canonical raw geo shape (round-3 F9): {lat, lon}
      // numbers, or {name}. "lat, lon" input becomes a point; anything
      // else becomes a named region.
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
 * Advisory issues for one field (empty list = no advisory finding). The
 * core's validation report is the authoritative one (I2).
 */
export function advisoryIssues(attr: FormAttr, raw: RawFieldValue): readonly string[] {
  const issues: string[] = [];
  const value = fieldValue(attr, raw);
  if (value === undefined) {
    if (attr.required) {
      issues.push("This field is required.");
    }
    if (
      attr.attrType === "number" &&
      typeof raw === "string" &&
      raw.trim().length > 0
    ) {
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

export type ConsentBlock = {
  readonly consent_text_digest: string;
  readonly consent_affirmed: boolean;
  readonly consent_affirmed_at: number;
};

export type PayloadInput = {
  readonly kind: string;
  readonly fields: Readonly<Record<string, RawFieldValue>>;
  readonly attributes: readonly FormAttr[];
  readonly consent: ConsentBlock;
  readonly submissionId: string;
  readonly formVersion: string;
  readonly capturedAtMs: number;
};

/**
 * Assembles the inner-payload-shaped submission object (blueprint section
 * 4): submission_version, app-generated submission_id, form_version,
 * consent block, captured_at, the payload-carried kind (D-070.5), and the
 * typed fields (empty optional fields omitted).
 */
export function buildPayload(input: PayloadInput): JsonObject {
  const fields: Record<string, JsonValue> = {};
  for (const attr of input.attributes) {
    const raw = input.fields[attr.id];
    if (raw === undefined) {
      continue;
    }
    const value = fieldValue(attr, raw);
    if (value !== undefined) {
      fields[attr.id] = value;
    }
  }
  return {
    submission_version: SUBMISSION_VERSION,
    submission_id: input.submissionId,
    form_version: input.formVersion,
    kind: input.kind,
    consent: { ...input.consent },
    captured_at: input.capturedAtMs,
    fields,
  };
}

/**
 * The structural consent gate (D-030): with the checkbox unchecked nothing
 * stages, regardless of field state. Advisory field issues also block
 * submit so obviously-broken payloads are caught before staging - the core
 * still re-validates authoritatively.
 */
export function canSubmit(
  attributes: readonly FormAttr[],
  fields: Readonly<Record<string, RawFieldValue>>,
  consentAffirmed: boolean,
): boolean {
  if (!consentAffirmed) {
    return false;
  }
  return attributes.every(
    (attr) => advisoryIssues(attr, fields[attr.id] ?? "").length === 0,
  );
}
