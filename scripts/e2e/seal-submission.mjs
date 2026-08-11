/**
 * Seal one SYNTHETIC remote intake submission using the form's REAL crypto path,
 * for the automated form-to-graph rehearsal (blueprint
 * docs/blueprints/intake-relay.md section 8 step 9; ADR-005 D3).
 *
 * This harness imports the form's real `form/src/crypto.ts` - the ACTUAL
 * `sodium.crypto_box_seal` call and the ORIGINAL/STANDARD base64 variant the
 * browser uses (D-083) - so the ciphertext this produces is sealed exactly as a
 * real browser submission would be, and the Rust puller opens it on the real
 * path. The trivial envelope JSON assembly (buildInnerPayload / buildOuterEnvelope)
 * is mirrored inline from `form/src/envelope.ts` rather than imported: envelope.ts
 * uses extensionless relative imports (`from "./crypto"`) that do not resolve
 * under plain Node ESM, and the shipped form source must not be edited to add
 * extensions (step 9 is additive). Any drift in the mirrored shape fails LOUDLY -
 * the puller's `OuterEnvelope::parse` / `InnerPayload::parse` reject a wrong
 * shape, so the e2e cannot pass on a mis-mirrored envelope. JS<->Rust crypto
 * equivalence itself is separately pinned by fixtures/crypto/sealed-box-vectors.json
 * (not re-proven here).
 *
 * Node-and-manual by construction (libsodium WASM); NOT a check-all member, the
 * same precedent as scripts/generate-crypto-vectors.js (blueprint section 1).
 *
 * PII posture: synthetic data only. The demo member is fictional and every email
 * is @example.test (I1). Nothing this writes is committed - the envelope lands in
 * a system temp dir the e2e cleans up.
 *
 * Usage:
 *   node scripts/e2e/seal-submission.mjs \
 *     --public <keygen public.json> --out <envelope.json> --digest <64-hex consent digest>
 *     [--form-version <tag>] [--display-name <text>]
 *
 * Prints a JSON summary to stdout: recipient_fingerprint, computed_fingerprint,
 * submission_id, display_name, consent_digest.
 */
import { readFileSync, writeFileSync } from "node:fs";
import { randomUUID } from "node:crypto";

// The form's REAL crypto boundary. `libsodium-wrappers` resolves from
// form/node_modules relative to crypto.ts. Node 24 strips the TS types natively.
import { seal, toBase64, hexToBytes, computeFingerprint, ready } from "../../form/src/crypto.ts";

/** envelope.rs SUBMISSION_VERSION / INTAKE_ENVELOPE_VERSION (form/src/envelope.ts). */
const SUBMISSION_VERSION = "0.1.0";
const INTAKE_ENVELOPE_VERSION = "0.1.0";

function parseArgs(argv) {
  const args = {
    public: null,
    out: null,
    digest: null,
    formVersion: "e2e-remote-rehearsal",
    displayName: "E2E-REMOTE Marisol Tidewater",
  };
  for (let i = 0; i < argv.length; i += 1) {
    const flag = argv[i];
    const value = argv[i + 1];
    switch (flag) {
      case "--public": args.public = value; i += 1; break;
      case "--out": args.out = value; i += 1; break;
      case "--digest": args.digest = value; i += 1; break;
      case "--form-version": args.formVersion = value; i += 1; break;
      case "--display-name": args.displayName = value; i += 1; break;
      default: throw new Error(`unknown argument '${flag}'`);
    }
  }
  for (const required of ["public", "out", "digest"]) {
    if (!args[required]) throw new Error(`--${required} is required`);
  }
  return args;
}

/**
 * Mirrors form/src/envelope.ts buildInnerPayload: STRING (ISO) timestamps, not
 * the fixtures' epoch numbers (the load-bearing remote/in-app difference). The
 * fisheries `person` kind fields are synthetic; `contact_email` is a mailto:
 * URI so cn-model's email-format validation accepts it (a bare address would
 * block the approval).
 */
function buildInnerPayload(args) {
  const nowIso = new Date().toISOString();
  return {
    submission_version: SUBMISSION_VERSION,
    submission_id: randomUUID(),
    form_version: args.formVersion,
    consent: {
      consent_text_digest: args.digest,
      consent_affirmed: true,
      consent_affirmed_at: nowIso,
    },
    captured_at: nowIso,
    kind: "person",
    fields: {
      display_name: args.displayName,
      contact_email: "mailto:marisol.tidewater@example.test",
      roles: ["harvest-coordinator", "youth-mentor"],
      seasons_active: "summer",
    },
  };
}

/** Mirrors form/src/envelope.ts buildOuterEnvelope, using the form's real seal(). */
function buildOuterEnvelope(inner, recipientPublicKey, recipientFingerprint) {
  const plaintext = new TextEncoder().encode(JSON.stringify(inner));
  const ciphertext = seal(plaintext, recipientPublicKey);
  return {
    intake_envelope_version: INTAKE_ENVELOPE_VERSION,
    recipient_key_fingerprint: recipientFingerprint,
    ciphertext: toBase64(ciphertext),
  };
}

async function main() {
  const args = parseArgs(process.argv.slice(2));
  await ready();

  // Read the keygen public key file (cn-ingest KeyFileEnvelope): hex public key
  // + its fingerprint.
  const keyFile = JSON.parse(readFileSync(args.public, "utf8"));
  const pubHex = keyFile.public_key;
  const fileFingerprint = keyFile.fingerprint;
  if (typeof pubHex !== "string" || typeof fileFingerprint !== "string") {
    throw new Error(`'${args.public}' is not a public key file (no public_key/fingerprint)`);
  }
  const recipientPublicKey = hexToBytes(pubHex);

  // Sanity: recompute the fingerprint from the raw key with the form's own
  // routine and confirm it equals the file's declared fingerprint (the same
  // BLAKE2b-truncated fingerprint the relay allowlist and puller pin use).
  const computed = computeFingerprint(recipientPublicKey);
  if (computed !== fileFingerprint) {
    throw new Error(
      `fingerprint mismatch: file says ${fileFingerprint}, recomputed ${computed}`,
    );
  }

  const inner = buildInnerPayload(args);
  const outer = buildOuterEnvelope(inner, recipientPublicKey, fileFingerprint);
  writeFileSync(args.out, JSON.stringify(outer));

  process.stdout.write(
    JSON.stringify({
      recipient_fingerprint: fileFingerprint,
      computed_fingerprint: computed,
      submission_id: inner.submission_id,
      display_name: inner.fields.display_name,
      consent_digest: args.digest,
      envelope_bytes: Buffer.byteLength(JSON.stringify(outer)),
    }) + "\n",
  );
}

main().catch((err) => {
  process.stderr.write(`seal-submission: error: ${err.message ?? err}\n`);
  process.exit(1);
});
