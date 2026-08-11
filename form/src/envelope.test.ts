import { beforeAll, describe, expect, it } from "vitest";

import { ready, fromBase64, hexToBytes } from "./crypto";
import {
  INTAKE_ENVELOPE_VERSION,
  SUBMISSION_VERSION,
  buildInnerPayload,
  buildOuterEnvelope,
} from "./envelope";

const TEST_PUBLIC_KEY_HEX = "205fd0dce7b409a3231b86dad10f6e3a276bb2e0838bc605501131f96b117a71";
const TEST_FINGERPRINT = "efdf-7ce7-69fa-feeb-7512-a100-6450-45c6";
const ISO_RE = /^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d{1,9})?Z$/;
const SEALED_BOX_OVERHEAD = 48;

function sampleInner() {
  return buildInnerPayload({
    submissionId: "00000000-0000-4000-8000-000000000000",
    formVersion: "remote-draft-2026-08-11",
    kind: "person",
    fields: { display_name: "Synthetic Person" },
    consentTextDigest: "ae5f7cc0d740726e81785919d1dbe6ad715895c94da278610da354d730bcd36c",
    consentAffirmed: true,
  });
}

describe("InnerPayload construction (must match cn-ingest envelope.rs InnerPayload)", () => {
  it("has exactly the documented field names", () => {
    const inner = sampleInner();
    expect(Object.keys(inner).sort()).toEqual(
      ["captured_at", "consent", "fields", "form_version", "kind", "submission_id", "submission_version"].sort(),
    );
    expect(Object.keys(inner.consent).sort()).toEqual(
      ["consent_affirmed", "consent_affirmed_at", "consent_text_digest"].sort(),
    );
  });

  it("uses the version constants from envelope.rs", () => {
    // envelope.rs SUBMISSION_VERSION / INTAKE_ENVELOPE_VERSION are both "0.1.0".
    expect(sampleInner().submission_version).toBe("0.1.0");
    expect(SUBMISSION_VERSION).toBe("0.1.0");
    expect(INTAKE_ENVELOPE_VERSION).toBe("0.1.0");
  });

  it("emits captured_at and consent_affirmed_at as ISO STRINGS, not numbers (regression)", () => {
    // envelope.rs: pub captured_at: String; pub consent_affirmed_at: String.
    // The in-app path emits these as numbers; the remote payload is parsed by
    // the strict Rust struct, where a number is a hard deserialize failure.
    const inner = sampleInner();
    expect(typeof inner.captured_at).toBe("string");
    expect(typeof inner.consent.consent_affirmed_at).toBe("string");
    expect(inner.captured_at).toMatch(ISO_RE);
    expect(inner.consent.consent_affirmed_at).toMatch(ISO_RE);
    // Explicitly NOT numbers.
    expect(typeof inner.captured_at as unknown).not.toBe("number");
  });

  it("carries consent_affirmed true and the payload-carried kind", () => {
    const inner = sampleInner();
    expect(inner.consent.consent_affirmed).toBe(true);
    expect(inner.kind).toBe("person");
  });

  it("honors injected timestamps when provided", () => {
    const inner = buildInnerPayload({
      submissionId: "id",
      formVersion: "v",
      kind: "person",
      fields: {},
      consentTextDigest: "d",
      consentAffirmed: true,
      capturedAt: "2026-08-11T00:00:00.000Z",
      consentAffirmedAt: "2026-08-11T00:00:01.000Z",
    });
    expect(inner.captured_at).toBe("2026-08-11T00:00:00.000Z");
    expect(inner.consent.consent_affirmed_at).toBe("2026-08-11T00:00:01.000Z");
  });
});

describe("OuterEnvelope construction (must match cn-ingest envelope.rs OuterEnvelope)", () => {
  beforeAll(async () => {
    await ready();
  });

  it("has exactly the documented fields and seals the inner payload", () => {
    const inner = sampleInner();
    const outer = buildOuterEnvelope(inner, hexToBytes(TEST_PUBLIC_KEY_HEX), TEST_FINGERPRINT);
    expect(Object.keys(outer).sort()).toEqual(
      ["ciphertext", "intake_envelope_version", "recipient_key_fingerprint"].sort(),
    );
    expect(outer.intake_envelope_version).toBe("0.1.0");
    expect(outer.recipient_key_fingerprint).toBe(TEST_FINGERPRINT);
    expect(typeof outer.ciphertext).toBe("string");

    // Ciphertext decodes to plaintext length + 48 (sealed-box overhead).
    const plaintextLen = new TextEncoder().encode(JSON.stringify(inner)).length;
    expect(fromBase64(outer.ciphertext).length).toBe(plaintextLen + SEALED_BOX_OVERHEAD);
  });
});

describe("e2e faithfulness: envelope.ts is the single source for the D-088 string convention (R5-2)", () => {
  // scripts/e2e/seal-submission.mjs MIRRORS buildInnerPayload inline rather than
  // importing it (the shipped form uses extensionless relative imports that do
  // not resolve under plain Node ESM). So a revert of captured_at /
  // consent_affirmed_at to epoch NUMBERS would still seal and pass the e2e,
  // failing only later at the Rust puller's InnerPayload::parse (both are
  // `String`). This pins the ISO-string convention at the REAL source
  // (form/src/envelope.ts) so such drift fails a form test first. It asserts
  // through the JSON.stringify boundary the seal path actually crosses - numbers
  // survive JSON serialization as numbers - matching what the puller decrypts.
  it("keeps captured_at and consent_affirmed_at as JSON strings across serialization", () => {
    const serialized = JSON.parse(JSON.stringify(buildInnerPayload({
      submissionId: "00000000-0000-4000-8000-000000000000",
      formVersion: "remote-draft-2026-08-11",
      kind: "person",
      fields: { display_name: "Synthetic Person" },
      consentTextDigest: "ae5f7cc0d740726e81785919d1dbe6ad715895c94da278610da354d730bcd36c",
      consentAffirmed: true,
    })));
    expect(typeof serialized.captured_at).toBe("string");
    expect(typeof serialized.consent.consent_affirmed_at).toBe("string");
  });
});
