/**
 * Build-time configuration, injected as compile-time constants by Vite's
 * `define` (see vite.config.ts). For local development and this step's tests
 * the values default to the TEST-ONLY committed keypair from
 * fixtures/crypto/sealed-box-vectors.json and a local wrangler-dev relay
 * origin. The real facilitator key and deployed relay origin are supplied at
 * build time via CN_FORM_* env vars (post-ceremony, out of scope here).
 *
 * These `declare const`s are replaced textually by Vite; this module is never
 * imported by the Node build scripts (which have no `define` pass), so the
 * bare identifiers never reach a plain-Node runtime. Vitest applies the same
 * `define` block, so config.test.ts may import this module.
 */
import type { JsonObject } from "./json";

declare const __CN_FORM_PUBLIC_KEY_HEX__: string;
declare const __CN_FORM_KEY_FINGERPRINT__: string;
declare const __CN_FORM_RELAY_ORIGIN__: string;
declare const __CN_FORM_VERSION__: string;
declare const __CN_FORM_TEMPLATE_JSON__: string;
declare const __CN_FORM_KINDS__: string;

/** Hex-encoded recipient X25519 public key (crypto.rs `encode_hex` format). */
export const PUBLIC_KEY_HEX: string = __CN_FORM_PUBLIC_KEY_HEX__;

/** Recipient key fingerprint (`efdf-7ce7-...`), rendered in the footer. */
export const KEY_FINGERPRINT: string = __CN_FORM_KEY_FINGERPRINT__;

/** The one CSP-allowed connect destination; where sealed envelopes POST. */
export const RELAY_ORIGIN: string = __CN_FORM_RELAY_ORIGIN__;

/** Provenance tag carried in InnerPayload.form_version. */
export const FORM_VERSION: string = __CN_FORM_VERSION__;

/** The group template baked in at build time; drives the R2 field widgets. */
export const TEMPLATE: JsonObject = JSON.parse(__CN_FORM_TEMPLATE_JSON__) as JsonObject;

/** The baked-in template's id (`template_id`), or "" when it carries none. */
export const TEMPLATE_ID: string =
  typeof TEMPLATE["template_id"] === "string" ? TEMPLATE["template_id"] : "";

// --- Kind restriction (CN_FORM_KINDS) ---------------------------------------

/**
 * Parses the comma-separated CN_FORM_KINDS build value into kind ids. Empty
 * or whitespace-only input means "no restriction" (every template kind).
 */
export function parseKindList(raw: string): readonly string[] {
  const seen = new Set<string>();
  const ids: string[] = [];
  for (const item of raw.split(",")) {
    const id = item.trim();
    if (id.length > 0 && !seen.has(id)) {
      seen.add(id);
      ids.push(id);
    }
  }
  return ids;
}

/**
 * Restricts a template's kinds to `allowed`, preserving the template's order.
 * An empty `allowed` list returns `kinds` unchanged (today's behavior). An id
 * the template does not define throws: vite.config.ts already fails the build
 * on that, and a silent empty form must never reach a participant.
 */
export function restrictKinds<K extends { readonly id: string }>(
  kinds: readonly K[],
  allowed: readonly string[],
): readonly K[] {
  if (allowed.length === 0) {
    return kinds;
  }
  const known = new Set(kinds.map((kind) => kind.id));
  for (const id of allowed) {
    if (!known.has(id)) {
      throw new Error(`kind restriction names unknown kind "${id}"`);
    }
  }
  const allowedSet = new Set(allowed);
  return kinds.filter((kind) => allowedSet.has(kind.id));
}

/** Kind ids the built form offers; empty = every kind in the template. */
export const ALLOWED_KINDS: readonly string[] = parseKindList(__CN_FORM_KINDS__);

// --- Per-template labels and help text ---------------------------------------

/**
 * Friendlier labels for attribute ids, keyed by template id and consulted by
 * the renderer before falling back to the raw id (the group-template schema
 * carries no human-readable label per attribute). Keyed per template so a
 * build against another template is unaffected; templates with no entry fall
 * back to raw ids. Community-facing wording: pending D-023 review, so the UI
 * keeps the DRAFT tag.
 */
export const FRIENDLY_LABELS_BY_TEMPLATE: Readonly<
  Record<string, Readonly<Record<string, string>>>
> = {
  "atni-convention": {
    display_name: "Name",
    tribe: "Tribal Nation or organization",
    role: "Role",
    areas_of_interest: "Priority areas",
    specialties: "Specialties",
    events_of_interest: "Events you plan to attend",
    contact_email: "Email",
    contact_preference: "How you prefer to be reached",
  },
};

/** Caption-style help text shown under a field, keyed like the labels. */
export const FIELD_HELP_BY_TEMPLATE: Readonly<
  Record<string, Readonly<Record<string, string>>>
> = {
  "atni-convention": {
    areas_of_interest: "One or two areas, one per line.",
  },
};

/** The label map for the baked-in template (empty when it has none). */
export const FRIENDLY_LABELS: Readonly<Record<string, string>> =
  FRIENDLY_LABELS_BY_TEMPLATE[TEMPLATE_ID] ?? {};

/** The help-text map for the baked-in template (empty when it has none). */
export const FIELD_HELP: Readonly<Record<string, string>> =
  FIELD_HELP_BY_TEMPLATE[TEMPLATE_ID] ?? {};
