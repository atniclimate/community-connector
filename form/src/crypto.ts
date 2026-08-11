/**
 * Browser-side crypto boundary for the remote intake form (ADR-005 D3;
 * blueprint sections 1 and 4). This is the JS half of a cross-implementation
 * boundary: what `seal()` produces here must open in the Rust puller
 * (core/crates/cn-ingest/src/crypto.rs), pinned by the committed test vectors
 * in fixtures/crypto/sealed-box-vectors.json.
 *
 * Algorithm: libsodium `crypto_box_seal` = anonymous sealed box
 * (X25519 + XSalsa20-Poly1305, BLAKE2b nonce). A fresh ephemeral keypair per
 * seal makes the sender anonymous and the ciphertext non-deterministic (see
 * crypto.rs `seal_is_anonymous_and_nondeterministic`) - which is why a retry
 * must re-POST already-sealed bytes rather than reseal (see submit.ts).
 *
 * Base64 variant: ORIGINAL (RFC 4648 s4, standard padded) - matches crypto.rs's
 * `base64::engine::general_purpose::STANDARD` used for its other base64 fields,
 * so the whole crypto surface uses one variant (DECISIONS.md D-083).
 */
import sodium from "libsodium-wrappers";

let readyPromise: Promise<void> | null = null;

/** Awaits libsodium WASM initialization (idempotent). */
export async function ready(): Promise<void> {
  readyPromise ??= sodium.ready;
  await readyPromise;
}

/** Decodes a lowercase/uppercase hex string to bytes (mirrors crypto.rs `decode_hex`). */
export function hexToBytes(hex: string): Uint8Array {
  if (hex.length % 2 !== 0) {
    throw new Error("hex string has an odd length");
  }
  const out = new Uint8Array(hex.length / 2);
  for (let i = 0; i < out.length; i += 1) {
    const byte = Number.parseInt(hex.slice(i * 2, i * 2 + 2), 16);
    if (Number.isNaN(byte)) {
      throw new Error("invalid hex character");
    }
    out[i] = byte;
  }
  return out;
}

/** Lowercase hex encoding (mirrors crypto.rs `encode_hex`). */
export function bytesToHex(bytes: Uint8Array): string {
  let out = "";
  for (const b of bytes) {
    out += b.toString(16).padStart(2, "0");
  }
  return out;
}

/**
 * Seals `plaintext` to `recipientPublicKey` as a libsodium anonymous sealed
 * box. Requires `ready()` first. Non-deterministic by construction (fresh
 * ephemeral key per call).
 */
export function seal(plaintext: Uint8Array, recipientPublicKey: Uint8Array): Uint8Array {
  return sodium.crypto_box_seal(plaintext, recipientPublicKey);
}

/** Standard padded base64 encode (ORIGINAL variant). */
export function toBase64(bytes: Uint8Array): string {
  return sodium.to_base64(bytes, sodium.base64_variants.ORIGINAL);
}

/** Standard padded base64 decode (ORIGINAL variant). */
export function fromBase64(text: string): Uint8Array {
  return sodium.from_base64(text, sodium.base64_variants.ORIGINAL);
}

/**
 * Computes a key fingerprint exactly as crypto.rs `fingerprint` and
 * scripts/generate-crypto-vectors.js `fingerprint`: BLAKE2b-256 over the raw
 * public key, truncated to 16 bytes, rendered as 8 lowercase hex groups of 4,
 * dash-separated (e.g. `efdf-7ce7-...`). Requires `ready()` first.
 */
export function computeFingerprint(publicKey: Uint8Array): string {
  const digest = sodium.crypto_generichash(32, publicKey);
  const hex = bytesToHex(digest.slice(0, 16));
  const groups = hex.match(/.{4}/g);
  if (groups === null) {
    throw new Error("fingerprint digest was too short");
  }
  return groups.join("-");
}
