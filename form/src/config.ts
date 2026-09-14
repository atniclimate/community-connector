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

// --- Per-template question order ---------------------------------------------

/**
 * The questions a template's form asks, per kind, as attribute ids in display
 * order. The list is also an ALLOWLIST: an attribute the template defines but
 * the list omits is neither rendered nor submitted (every omitted attribute is
 * optional in the template, so the core's validation still passes). A kind or
 * template with no entry keeps today's behavior: every attribute, in template
 * order. An id the template does not define is a configuration error and
 * `orderAttributes` throws, so it surfaces as the fatal panel on first render
 * (and fails config.test.ts against the template on disk), never as a
 * silently shorter form.
 *
 * atni-convention: the nine convention questions the human wrote on
 * 2026-09-14 (docs/design/intake-questions-2026-09-14.md), in their order.
 * tribe, role, events_of_interest, contact_email, and contact_preference stay
 * in the template for the fixture and the in-app facilitator form but are
 * not asked here.
 */
export const QUESTION_ORDER_BY_TEMPLATE: Readonly<
  Record<string, Readonly<Record<string, readonly string[]>>>
> = {
  "atni-convention": {
    person: [
      "display_name",
      "roles",
      "origins",
      "areas_of_interest",
      "specialties",
      "connections",
      "seeking",
      "offering",
      "committee_memberships",
    ],
  },
};

/**
 * Applies a question order to a kind's attributes: the listed attributes, in
 * the listed order, and nothing else. An undefined `order` returns
 * `attributes` unchanged. A listed id the kind does not define throws.
 */
export function orderAttributes<A extends { readonly id: string }>(
  attributes: readonly A[],
  order: readonly string[] | undefined,
): readonly A[] {
  if (order === undefined) {
    return attributes;
  }
  const byId = new Map(attributes.map((attr) => [attr.id, attr] as const));
  return order.map((id) => {
    const attr = byId.get(id);
    if (attr === undefined) {
      throw new Error(`question order names unknown attribute "${id}"`);
    }
    return attr;
  });
}

// --- Per-template labels and help text ---------------------------------------

/**
 * Labels for attribute ids, keyed by template id and consulted by the renderer
 * before falling back to the raw id (the group-template schema carries no
 * human-readable label per attribute). Keyed per template so a build against
 * another template is unaffected; templates with no entry fall back to raw
 * ids. Community-facing wording: pending D-023 review, so the UI keeps the
 * DRAFT tag.
 *
 * atni-convention: the human's question wording, verbatim, including the
 * capitalization and the "(Optional)" marker on the connections question.
 * The renderer appends " (required)" to the required name question.
 */
export const FRIENDLY_LABELS_BY_TEMPLATE: Readonly<
  Record<string, Readonly<Record<string, string>>>
> = {
  "atni-convention": {
    display_name: "What is your Name?",
    roles: "What do you Do?",
    origins: "Where are you from?",
    areas_of_interest: "What is important to you?",
    specialties: "What are you good at?",
    connections: "Who are you connected with? (Optional)",
    seeking: "What are you hoping to find?",
    offering: "What are you here to share?",
    committee_memberships: "What committees will you attend?",
  },
};

/**
 * Caption-style help text shown under a field, keyed like the labels. The
 * atni-convention entries are the human's help wording, verbatim, trailing
 * ellipses included.
 */
export const FIELD_HELP_BY_TEMPLATE: Readonly<
  Record<string, Readonly<Record<string, string>>>
> = {
  "atni-convention": {
    display_name: "How do you prefer to be addressed?",
    roles:
      "Share as many roles, positions, or areas of responsibility as feel right; " +
      "a single title rarely tells the whole story...",
    origins: "The Tribe(s), Places, Communities, and Organizations that bring you here...",
    areas_of_interest:
      "The issues, committees, and areas you devote your time, energy, and thinking to... " +
      "(e.g. climate, healthcare, human rights, etc.)",
    specialties:
      "The things people come to you for, whether or not they show up in a job description...",
    connections:
      "The people, communities, and organizations you carry with you; " +
      "the ones that stay on your mind when you think about this work...",
    seeking:
      "The connection you came here looking for; a collaborator, a conversation, " +
      "someone doing similar work, or something you haven't found yet...",
    offering:
      "Your presence matters. The Knowledge, experience, opportunities, or gifts " +
      "you bring into this room...",
    committee_memberships:
      "ATNI committees, working groups, or sessions you plan to participate in...",
  },
};

/** The per-kind question order for the baked-in template (empty when it has none). */
export const QUESTION_ORDER: Readonly<Record<string, readonly string[]>> =
  QUESTION_ORDER_BY_TEMPLATE[TEMPLATE_ID] ?? {};

/** The label map for the baked-in template (empty when it has none). */
export const FRIENDLY_LABELS: Readonly<Record<string, string>> =
  FRIENDLY_LABELS_BY_TEMPLATE[TEMPLATE_ID] ?? {};

/** The help-text map for the baked-in template (empty when it has none). */
export const FIELD_HELP: Readonly<Record<string, string>> =
  FIELD_HELP_BY_TEMPLATE[TEMPLATE_ID] ?? {};
