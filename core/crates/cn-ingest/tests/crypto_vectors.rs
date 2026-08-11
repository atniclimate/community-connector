//! Cross-implementation sealed-box interop (blueprint intake-relay section 1;
//! ADR-005 D3). The browser seals with libsodium.js; this binding opens with
//! RustCrypto's `crypto_box`. The committed fixture
//! `fixtures/crypto/sealed-box-vectors.json` is produced by libsodium.js
//! (scripts/generate-crypto-vectors.js); these tests are the interop pin:
//! every libsodium-sealed box MUST open here, and the fingerprints MUST match.
//! Regenerate the fixture with `npm --prefix scripts run generate:crypto-vectors`.

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64;
use cn_ingest::{Keypair, PublicKey, SecretKey, fingerprint, open, seal};
use serde_json::Value;

fn load_fixture() -> Value {
    // From this crate's dir up three levels to the repo root.
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../fixtures/crypto/sealed-box-vectors.json"
    );
    let bytes = std::fs::read(path).unwrap_or_else(|e| panic!("read fixture {path}: {e}"));
    serde_json::from_slice(&bytes).expect("parse fixture json")
}

fn hex_to_vec(s: &str) -> Vec<u8> {
    assert!(s.len().is_multiple_of(2), "hex string has even length");
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).expect("valid hex"))
        .collect()
}

fn arr32(bytes: &[u8]) -> [u8; 32] {
    bytes.try_into().expect("exactly 32 bytes")
}

fn fixture_keypair(f: &Value) -> Keypair {
    let kp = &f["keypair"];
    let public = PublicKey::from_bytes(arr32(&hex_to_vec(
        kp["public_key_hex"].as_str().expect("public_key_hex"),
    )));
    let secret = SecretKey::from_bytes(arr32(&hex_to_vec(
        kp["secret_key_hex"].as_str().expect("secret_key_hex"),
    )));
    Keypair { public, secret }
}

/// The fingerprint the Rust binding computes for the fixture public key must
/// equal the one libsodium.js computed - a cross-implementation known-key ->
/// known-fingerprint stability check.
#[test]
fn fixture_fingerprint_matches_libsodium() {
    let f = load_fixture();
    let kp = fixture_keypair(&f);
    let expected = f["keypair"]["fingerprint"].as_str().expect("fingerprint");
    assert_eq!(fingerprint(&kp.public).to_string(), expected);
}

/// libsodium's secret key must derive the same X25519 public key in Rust
/// (interop of the base-point multiply, not just the box).
#[test]
fn fixture_secret_derives_fixture_public() {
    let f = load_fixture();
    let kp = fixture_keypair(&f);
    assert_eq!(kp.secret.public_key(), kp.public);
}

/// Every libsodium.js `crypto_box_seal` box in the fixture opens in Rust to
/// the exact plaintext.
#[test]
fn libsodium_sealed_boxes_open_in_rust() {
    let f = load_fixture();
    let kp = fixture_keypair(&f);
    let vectors = f["seal_vectors"].as_array().expect("seal_vectors");
    assert!(!vectors.is_empty(), "fixture has vectors");
    for v in vectors {
        let description = v["description"].as_str().unwrap_or("<unnamed>");
        let ciphertext = BASE64
            .decode(v["ciphertext_base64"].as_str().expect("ciphertext_base64"))
            .expect("valid base64");
        let expected = match v["plaintext_encoding"].as_str().expect("encoding") {
            "hex" => hex_to_vec(v["plaintext"].as_str().expect("plaintext")),
            "utf8" => v["plaintext"]
                .as_str()
                .expect("plaintext")
                .as_bytes()
                .to_vec(),
            other => panic!("unknown plaintext encoding {other}"),
        };
        let opened =
            open(&ciphertext, &kp).unwrap_or_else(|_| panic!("open failed: {description}"));
        assert_eq!(opened, expected, "vector {description}");
    }
}

/// Round trip the other direction: Rust seals to the fixture public key and
/// opens with the fixture keypair.
#[test]
fn rust_sealed_box_opens_with_fixture_key() {
    let f = load_fixture();
    let kp = fixture_keypair(&f);
    let msg = b"round trip through the rust binding";
    let ciphertext = seal(msg, &kp.public);
    assert_eq!(open(&ciphertext, &kp).expect("open"), msg);
}

/// Malformed ciphertext (truncation, tag flip, ephemeral-key flip) fails; the
/// error is content-free.
#[test]
fn tampered_fixture_ciphertext_fails() {
    let f = load_fixture();
    let kp = fixture_keypair(&f);
    let vectors = f["seal_vectors"].as_array().expect("seal_vectors");
    let v = vectors
        .iter()
        .find(|v| v["description"].as_str() == Some("inner-payload-shaped json"))
        .expect("json vector present");
    let ciphertext = BASE64
        .decode(v["ciphertext_base64"].as_str().expect("ciphertext_base64"))
        .expect("valid base64");

    // Truncated (drops part of the Poly1305 tag / body).
    assert!(open(&ciphertext[..ciphertext.len() - 1], &kp).is_err());

    // Bit-flip in the trailing bytes.
    let mut flipped = ciphertext.clone();
    let last = flipped.len() - 1;
    flipped[last] ^= 0x01;
    assert!(open(&flipped, &kp).is_err());

    // Bit-flip in the ephemeral public-key prefix (first 32 bytes).
    let mut head = ciphertext.clone();
    head[0] ^= 0x01;
    assert!(open(&head, &kp).is_err());
}
