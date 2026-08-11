/**
 * Envelope construction: builds the InnerPayload and OuterEnvelope JSON that
 * core/crates/cn-ingest/src/envelope.rs accepts via ordinary
 * serde_json::from_slice. Field names, JSON types, and version-string values
 * must match that Rust struct exactly; each field below cross-references its
 * authoritative line/type in envelope.rs (as documentation and a review aid -
 * full cross-language interop verification is blueprint step 9's job).
 *
 * LOAD-BEARING: `captured_at` and `consent.consent_affirmed_at` are Rust
 * `String` fields (envelope.rs pub captured_at: String ~line 109; pub
 * consent_affirmed_at: String ~line 148, both "Client ISO timestamp"). They
 * MUST be JSON strings (new Date().toISOString()), never epoch numbers. The
 * closest prior art (app/src/ui/forms/model.ts buildPayload) emits both as JSON
 * numbers - fine there because in-app records store as untyped serde_json::Value
 * and never hit InnerPayload::parse; here a number would be a hard deserialize
 * failure. Do NOT copy the in-app timestamp convention.
 */
import type { JsonObject, JsonValue } from "./json";
import { seal, toBase64 } from "./crypto";

/** envelope.rs SUBMISSION_VERSION (~line 34). Read from source if it bumps. */
export const SUBMISSION_VERSION = "0.1.0";

/** envelope.rs INTAKE_ENVELOPE_VERSION (~line 31). Independent version line. */
export const INTAKE_ENVELOPE_VERSION = "0.1.0";

/** ConsentBlock - envelope.rs ConsentBlock (~lines 141-149). No extras by design. */
export type ConsentBlock = {
  /** consent_text_digest: String (~line 143). */
  readonly consent_text_digest: string;
  /** consent_affirmed: bool, MUST be true to stage (~line 146, D-030). */
  readonly consent_affirmed: boolean;
  /** consent_affirmed_at: String, "Client ISO timestamp" (~line 148). */
  readonly consent_affirmed_at: string;
};

/** InnerPayload - envelope.rs InnerPayload (~lines 97-119). Sealed as plaintext. */
export type InnerPayload = {
  /** submission_version: semver::Version, deserialized from a JSON string (~line 99). */
  readonly submission_version: string;
  /** submission_id: String, client UUID, the semantic dedup key (~line 101). */
  readonly submission_id: string;
  /** form_version: String (~line 103). */
  readonly form_version: string;
  /** consent: ConsentBlock (~line 105). */
  readonly consent: ConsentBlock;
  /** captured_at: String, "Client ISO timestamp" (~line 109) - a STRING, not a number. */
  readonly captured_at: string;
  /** kind: Option<String> (~line 113); always present here (the form always picks a kind). */
  readonly kind: string;
  /** fields: BTreeMap<String, Value> (~line 115). */
  readonly fields: JsonObject;
};

/** OuterEnvelope - envelope.rs OuterEnvelope (~lines 60-73). POSTed to /submit. */
export type OuterEnvelope = {
  /** intake_envelope_version: semver::Version from a JSON string (~line 63). */
  readonly intake_envelope_version: string;
  /** recipient_key_fingerprint: String, a routing hint only (~line 66). */
  readonly recipient_key_fingerprint: string;
  /** ciphertext: String, base64 of the sealed box - opaque to the relay (~line 69). */
  readonly ciphertext: string;
};

export type InnerPayloadInput = {
  readonly submissionId: string;
  readonly formVersion: string;
  readonly kind: string;
  readonly fields: Readonly<Record<string, JsonValue>>;
  readonly consentTextDigest: string;
  readonly consentAffirmed: boolean;
  /** ISO-8601 instants (Date.toISOString()); defaults to now for each. */
  readonly capturedAt?: string;
  readonly consentAffirmedAt?: string;
};

/** Builds the InnerPayload object. Timestamps are ISO strings (see file doc). */
export function buildInnerPayload(input: InnerPayloadInput): InnerPayload {
  const nowIso = new Date().toISOString();
  return {
    submission_version: SUBMISSION_VERSION,
    submission_id: input.submissionId,
    form_version: input.formVersion,
    consent: {
      consent_text_digest: input.consentTextDigest,
      consent_affirmed: input.consentAffirmed,
      consent_affirmed_at: input.consentAffirmedAt ?? nowIso,
    },
    captured_at: input.capturedAt ?? nowIso,
    kind: input.kind,
    fields: { ...input.fields },
  };
}

/**
 * Seals an InnerPayload to `recipientPublicKey` and wraps it in an
 * OuterEnvelope. The plaintext is ordinary JSON.stringify (UTF-8) - no
 * canonical/sorted-key serialization is needed because InnerPayload::parse does
 * plain object-shape parsing after decryption (canonical serialization matters
 * for HASHING elsewhere, not here). Sealing is non-deterministic; call this
 * exactly once per submit attempt and reuse the returned object on retry
 * (see submit.ts).
 */
export function buildOuterEnvelope(
  inner: InnerPayload,
  recipientPublicKey: Uint8Array,
  recipientFingerprint: string,
): OuterEnvelope {
  const plaintext = new TextEncoder().encode(JSON.stringify(inner));
  const ciphertext = seal(plaintext, recipientPublicKey);
  return {
    intake_envelope_version: INTAKE_ENVELOPE_VERSION,
    recipient_key_fingerprint: recipientFingerprint,
    ciphertext: toBase64(ciphertext),
  };
}
