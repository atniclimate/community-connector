import { beforeAll, describe, expect, it } from "vitest";

import { bytesToHex, computeFingerprint, fromBase64, hexToBytes, ready, seal, toBase64 } from "./crypto";

// TEST-ONLY committed keypair (public half + fingerprint) from
// fixtures/crypto/sealed-box-vectors.json. Never operational.
const TEST_PUBLIC_KEY_HEX = "205fd0dce7b409a3231b86dad10f6e3a276bb2e0838bc605501131f96b117a71";
const TEST_FINGERPRINT = "efdf-7ce7-69fa-feeb-7512-a100-6450-45c6";
// "short ascii" vector: crypto_box_seal("hello relay") base64 (standard padded).
const FIXTURE_SHORT_ASCII_B64 =
  "hQZBE7auU5sjJp7OlmYJL3RrsSvHxcDUEoGyV3ASfTmb4AI1XyacC5NGf6368vRRXaqr12zhYxYEV+A=";
const SEALED_BOX_OVERHEAD = 48; // 32-byte ephemeral public key + 16-byte Poly1305 tag

beforeAll(async () => {
  await ready();
});

describe("hex codec", () => {
  it("round-trips the test public key", () => {
    expect(bytesToHex(hexToBytes(TEST_PUBLIC_KEY_HEX))).toBe(TEST_PUBLIC_KEY_HEX);
  });

  it("rejects odd-length and non-hex input", () => {
    expect(() => hexToBytes("abc")).toThrow();
    expect(() => hexToBytes("zz")).toThrow();
  });
});

describe("fingerprint self-consistency", () => {
  it("recomputes the embedded fingerprint from the embedded public key", () => {
    const publicKey = hexToBytes(TEST_PUBLIC_KEY_HEX);
    expect(computeFingerprint(publicKey)).toBe(TEST_FINGERPRINT);
  });

  it("renders 8 dash-separated hex groups of 4", () => {
    const fp = computeFingerprint(hexToBytes(TEST_PUBLIC_KEY_HEX));
    const groups = fp.split("-");
    expect(groups).toHaveLength(8);
    expect(groups.every((g) => /^[0-9a-f]{4}$/.test(g))).toBe(true);
  });
});

describe("sealed-box wrapper", () => {
  it("base64 (ORIGINAL) round-trips", () => {
    const bytes = new Uint8Array([0, 1, 2, 250, 251, 252, 255]);
    expect(Array.from(fromBase64(toBase64(bytes)))).toEqual(Array.from(bytes));
  });

  it("produces ciphertext of plaintext length + 48 (sealed-box overhead)", () => {
    const publicKey = hexToBytes(TEST_PUBLIC_KEY_HEX);
    const plaintext = new TextEncoder().encode("synthetic inner payload");
    const ciphertext = seal(plaintext, publicKey);
    expect(ciphertext.length).toBe(plaintext.length + SEALED_BOX_OVERHEAD);
  });

  it("decodes a committed standard-base64 fixture ciphertext to the right length", () => {
    // Cross-checks that ORIGINAL == the generator's standard padded base64:
    // "hello relay" is 11 bytes, so the sealed box is 11 + 48 = 59 bytes.
    const raw = fromBase64(FIXTURE_SHORT_ASCII_B64);
    expect(raw.length).toBe("hello relay".length + SEALED_BOX_OVERHEAD);
  });

  it("is non-deterministic (fresh ephemeral key per seal)", () => {
    const publicKey = hexToBytes(TEST_PUBLIC_KEY_HEX);
    const plaintext = new TextEncoder().encode("same plaintext");
    const a = toBase64(seal(plaintext, publicKey));
    const b = toBase64(seal(plaintext, publicKey));
    expect(a).not.toBe(b);
  });
});
