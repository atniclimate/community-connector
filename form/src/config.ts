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
 * bare identifiers never reach a plain-Node runtime.
 */
import type { JsonObject } from "./json";

declare const __CN_FORM_PUBLIC_KEY_HEX__: string;
declare const __CN_FORM_KEY_FINGERPRINT__: string;
declare const __CN_FORM_RELAY_ORIGIN__: string;
declare const __CN_FORM_VERSION__: string;
declare const __CN_FORM_TEMPLATE_JSON__: string;

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

/**
 * Optional friendlier labels for attribute ids, consulted by the renderer
 * before falling back to the raw id (the group-template schema carries no
 * human-readable label per attribute). Empty by default and deliberately so:
 * neither fixture template is the confirmed deployment target, so inventing
 * plausible ids here risks drifting from the real pilot template. A later step
 * populates this once a real template is chosen.
 */
export const FRIENDLY_LABELS: Readonly<Record<string, string>> = {};
