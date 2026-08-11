//! Sealed-box crypto binding for the remote intake relay (blueprint
//! docs/blueprints/intake-relay.md section 1; ADR-005 D3).
//!
//! This is the Rust half of a cross-implementation crypto boundary: the
//! attendee's browser seals a submission with libsodium.js `crypto_box_seal`,
//! and the pilot-PC puller opens it here. The binding is RustCrypto's
//! `crypto_box` (`seal` feature) - pure Rust, no C build dependency - which
//! implements libsodium's anonymous sealed box (X25519 + XSalsa20-Poly1305,
//! BLAKE2b nonce). Interop is not assumed: it is pinned by the committed
//! cross-implementation test vectors in `fixtures/crypto/sealed-box-vectors.json`
//! (a libsodium.js-produced fixture that MUST open here). See D-081 for the
//! binding selection.
//!
//! The module also models the facilitator key-file envelopes from the keygen
//! ceremony (docs/design/facilitator-keygen-ceremony.md section 2): a `public`
//! envelope carrying the raw public key, and a `secret-encrypted` envelope
//! carrying the secret key under an Argon2id-derived XSalsa20-Poly1305
//! secretbox. Every artifact is a versioned JSON envelope; unknown MAJOR is
//! rejected loudly (I7).
//!
//! No file or network I/O lives here (ADR-005 D1 module fence): these are pure
//! crypto operations and byte-level (de)serialization. Callers (the `cn` CLI)
//! own reading and writing the bytes.

use std::collections::BTreeMap;
use std::fmt;
use std::str::FromStr;

use argon2::{Algorithm, Argon2, Params, Version};
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64;
use blake2::Blake2b;
use blake2::Digest as _;
use blake2::digest::consts::U32;
use crypto_box::aead::OsRng;
use crypto_box::aead::rand_core::RngCore as _;
use crypto_secretbox::aead::{Aead, KeyInit};
use crypto_secretbox::{Nonce, XSalsa20Poly1305};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::version::IngestError;

/// Raw X25519 public-key length.
pub const PUBLIC_KEY_LEN: usize = 32;
/// Raw X25519 secret-key length.
pub const SECRET_KEY_LEN: usize = 32;
/// Key fingerprint length (BLAKE2b-256 truncated to 128 bits).
pub const FINGERPRINT_LEN: usize = 16;
/// XSalsa20-Poly1305 secretbox nonce length.
const SECRETBOX_NONCE_LEN: usize = 24;
/// Argon2id salt length used when writing a secret-key envelope.
const ARGON2_SALT_LEN: usize = 16;

/// Key-file envelope format tag (ceremony design section 2).
pub const KEY_FILE_FORMAT: &str = "cn-intake-key";
/// Current key-file envelope schema version (semver; unknown MAJOR rejected).
pub const KEY_FILE_SCHEMA_VERSION: &str = "0.1.0";
/// KDF identifier stored in a `secret-encrypted` envelope.
const KDF_ARGON2ID: &str = "argon2id";
/// Cipher identifier stored in a `secret-encrypted` envelope.
const CIPHER_XSALSA20POLY1305: &str = "xsalsa20poly1305";

/// Argon2id parameters for writing a fresh secret-key envelope. OWASP-minimum
/// class (19 MiB, two passes); the exact values are stored in the envelope, so
/// a reader always uses what a writer recorded rather than these constants.
const DEFAULT_ARGON2_M_COST: u32 = 19_456;
const DEFAULT_ARGON2_T_COST: u32 = 2;
const DEFAULT_ARGON2_P_COST: u32 = 1;

/// A recipient's raw X25519 public key (non-secret; safe to log and commit).
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct PublicKey([u8; PUBLIC_KEY_LEN]);

impl PublicKey {
    /// Wraps raw public-key bytes.
    pub fn from_bytes(bytes: [u8; PUBLIC_KEY_LEN]) -> Self {
        Self(bytes)
    }

    /// Borrows the raw public-key bytes.
    pub fn as_bytes(&self) -> &[u8; PUBLIC_KEY_LEN] {
        &self.0
    }
}

impl fmt::Debug for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // The public key is not secret; show its fingerprint for readability.
        write!(f, "PublicKey({})", fingerprint(self))
    }
}

/// A raw X25519 secret key. Zeroized on drop so a decrypted key does not
/// linger in memory after the puller finishes (blueprint section 1).
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct SecretKey([u8; SECRET_KEY_LEN]);

impl SecretKey {
    /// Wraps raw secret-key bytes.
    pub fn from_bytes(bytes: [u8; SECRET_KEY_LEN]) -> Self {
        Self(bytes)
    }

    /// Borrows the raw secret-key bytes. Named to make call sites conspicuous:
    /// every use widens the window in which key material is copied.
    pub fn expose_bytes(&self) -> &[u8; SECRET_KEY_LEN] {
        &self.0
    }

    /// Derives the corresponding public key (X25519 base-point multiply).
    pub fn public_key(&self) -> PublicKey {
        let sk = crypto_box::SecretKey::from_bytes(self.0);
        PublicKey(*sk.public_key().as_bytes())
    }
}

impl fmt::Debug for SecretKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Never render secret bytes.
        f.write_str("SecretKey(<redacted>)")
    }
}

/// An X25519 keypair.
#[derive(Debug)]
pub struct Keypair {
    pub public: PublicKey,
    pub secret: SecretKey,
}

/// A key fingerprint: BLAKE2b-256 over the raw 32-byte public key, truncated
/// to 16 bytes. The human-verifiable and machine-pinned key identity
/// (ceremony design section 2).
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Fingerprint([u8; FINGERPRINT_LEN]);

impl Fingerprint {
    /// Borrows the raw fingerprint bytes.
    pub fn as_bytes(&self) -> &[u8; FINGERPRINT_LEN] {
        &self.0
    }
}

impl fmt::Display for Fingerprint {
    /// Renders `3f9a-1c02-...`: 8 lowercase hex groups of 4, dash-separated.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, pair) in self.0.chunks(2).enumerate() {
            if i > 0 {
                f.write_str("-")?;
            }
            write!(f, "{:02x}{:02x}", pair[0], pair[1])?;
        }
        Ok(())
    }
}

impl fmt::Debug for Fingerprint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Fingerprint({self})")
    }
}

impl FromStr for Fingerprint {
    type Err = IngestError;

    /// Parses the canonical `3f9a-1c02-...` form back to bytes: exactly 8
    /// dash-separated groups of 4 hex chars. Strict, so a mistyped pin fails
    /// loudly (I3).
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let groups: Vec<&str> = s.split('-').collect();
        if groups.len() != FINGERPRINT_LEN / 2 || groups.iter().any(|g| g.len() != 4) {
            return Err(IngestError::KeyFile {
                detail: "fingerprint must be 8 dash-separated groups of 4 hex characters".into(),
            });
        }
        let hex: String = groups.concat();
        let bytes = decode_hex(&hex)?;
        let mut fp = [0u8; FINGERPRINT_LEN];
        fp.copy_from_slice(&bytes);
        Ok(Fingerprint(fp))
    }
}

/// Generates a fresh X25519 keypair from the OS CSPRNG.
pub fn generate_keypair() -> Keypair {
    let sk = crypto_box::SecretKey::generate(&mut OsRng);
    let public = PublicKey(*sk.public_key().as_bytes());
    let secret = SecretKey(sk.to_bytes());
    Keypair { public, secret }
}

/// BLAKE2b-256 over the raw 32-byte public key, truncated to 16 bytes
/// (ceremony design section 2).
pub fn fingerprint(public: &PublicKey) -> Fingerprint {
    let mut hasher = Blake2b::<U32>::new();
    hasher.update(public.as_bytes());
    let digest = hasher.finalize();
    let mut fp = [0u8; FINGERPRINT_LEN];
    fp.copy_from_slice(&digest[..FINGERPRINT_LEN]);
    Fingerprint(fp)
}

/// Seals `plaintext` to `recipient` as a libsodium-compatible anonymous sealed
/// box (`crypto_box_seal`). The ephemeral secret key is generated internally
/// and discarded, so the sender is anonymous to the recipient.
///
/// Panics only if the OS CSPRNG fails, which is an unrecoverable environment
/// fault (a loud stop, never a silent wrong result - I3).
pub fn seal(plaintext: &[u8], recipient: &PublicKey) -> Vec<u8> {
    let pk = crypto_box::PublicKey::from_bytes(recipient.0);
    pk.seal(&mut OsRng, plaintext)
        .expect("sealed-box encryption is infallible with a working CSPRNG")
}

/// Opens a sealed box addressed to `recipient`. The error is content-free: a
/// failed open (wrong key, truncation, tampering) reveals nothing about why
/// (blueprint section 1).
pub fn open(ciphertext: &[u8], recipient: &Keypair) -> Result<Vec<u8>, IngestError> {
    let sk = crypto_box::SecretKey::from_bytes(recipient.secret.0);
    sk.unseal(ciphertext).map_err(|_| IngestError::Crypto)
}

// --- Key-file envelopes ------------------------------------------------------

/// Envelope role. `public` carries the raw public key; `secret-encrypted`
/// carries the secret key under a passphrase.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum KeyRole {
    Public,
    SecretEncrypted,
}

/// Argon2id parameters recorded alongside an encrypted secret key so a reader
/// can reproduce the derivation exactly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Argon2Params {
    pub m_cost: u32,
    pub t_cost: u32,
    pub p_cost: u32,
}

/// The `secret-encrypted` payload block: an Argon2id-derived XSalsa20-Poly1305
/// secretbox of the raw secret key, with the KDF/cipher identifiers and all
/// public parameters needed to reverse it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecretEnvelope {
    pub kdf: String,
    pub argon2: Argon2Params,
    /// Base64 Argon2id salt.
    pub salt: String,
    pub cipher: String,
    /// Base64 secretbox nonce.
    pub nonce: String,
    /// Base64 secretbox ciphertext (secret key + Poly1305 tag).
    pub ciphertext: String,
}

/// Caller-supplied metadata for a written key file. cn-ingest has no clock or
/// I/O; the caller stamps `created_at`.
#[derive(Debug, Clone)]
pub struct KeyMetadata {
    /// Creation timestamp (ISO-8601 by convention).
    pub created_at: String,
}

/// A parsed key-file envelope (ceremony design section 2). `public_key` and
/// `fingerprint` are present in both roles; `secret` only in
/// `secret-encrypted`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyFileEnvelope {
    pub format: String,
    pub schema_version: semver::Version,
    pub role: KeyRole,
    pub created_at: String,
    pub fingerprint: String,
    /// Hex-encoded raw public key (non-secret, present in both roles).
    pub public_key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub secret: Option<SecretEnvelope>,
    /// Unknown-minor fields preserved across read/write (I7).
    #[serde(flatten)]
    pub extras: BTreeMap<String, Value>,
}

fn key_file_current_version() -> semver::Version {
    semver::Version::parse(KEY_FILE_SCHEMA_VERSION).expect("valid const")
}

/// Rejects an unknown MAJOR key-file schema version loudly (I7); unknown minor
/// is tolerated (extras preserve it).
fn check_key_file_major(version: &semver::Version) -> Result<(), IngestError> {
    if version.major != key_file_current_version().major {
        return Err(IngestError::UnknownMajorVersion {
            found: version.clone(),
        });
    }
    Ok(())
}

/// Structural validation shared by both parsers: format tag, version major,
/// expected role, and the fingerprint-binds-the-public-key invariant.
fn validate_envelope(env: &KeyFileEnvelope, expected: KeyRole) -> Result<PublicKey, IngestError> {
    if env.format != KEY_FILE_FORMAT {
        return Err(IngestError::KeyFile {
            detail: format!("unexpected key-file format tag {:?}", env.format),
        });
    }
    check_key_file_major(&env.schema_version)?;
    if env.role != expected {
        return Err(IngestError::KeyFile {
            detail: format!("expected {expected:?} key file, found {:?}", env.role),
        });
    }
    let public = public_key_from_hex(&env.public_key)?;
    // The fingerprint must actually be the fingerprint of the stated public
    // key - the envelope cannot claim a fingerprint it does not own.
    let declared: Fingerprint = env.fingerprint.parse()?;
    if declared != fingerprint(&public) {
        return Err(IngestError::KeyFile {
            detail: "fingerprint does not match the public key".into(),
        });
    }
    Ok(public)
}

/// Parses a `public` key-file envelope, returning the verified public key.
pub fn parse_public_key(bytes: &[u8]) -> Result<PublicKey, IngestError> {
    let env: KeyFileEnvelope =
        serde_json::from_slice(bytes).map_err(|e| IngestError::Serialize(e.to_string()))?;
    validate_envelope(&env, KeyRole::Public)
}

/// Parses a `secret-encrypted` key-file envelope, deriving the passphrase key
/// with Argon2id and opening the XSalsa20-Poly1305 secretbox. A wrong
/// passphrase (or any tampering) fails content-free (`IngestError::Crypto`).
pub fn parse_secret_key(bytes: &[u8], passphrase: &str) -> Result<SecretKey, IngestError> {
    let env: KeyFileEnvelope =
        serde_json::from_slice(bytes).map_err(|e| IngestError::Serialize(e.to_string()))?;
    let public = validate_envelope(&env, KeyRole::SecretEncrypted)?;
    let sec = env.secret.as_ref().ok_or_else(|| IngestError::KeyFile {
        detail: "secret-encrypted envelope is missing its secret block".into(),
    })?;
    if sec.kdf != KDF_ARGON2ID {
        return Err(IngestError::KeyFile {
            detail: format!("unsupported kdf {:?}", sec.kdf),
        });
    }
    if sec.cipher != CIPHER_XSALSA20POLY1305 {
        return Err(IngestError::KeyFile {
            detail: format!("unsupported cipher {:?}", sec.cipher),
        });
    }

    let salt = BASE64.decode(&sec.salt).map_err(|_| IngestError::KeyFile {
        detail: "salt is not valid base64".into(),
    })?;
    let nonce_bytes = BASE64
        .decode(&sec.nonce)
        .map_err(|_| IngestError::KeyFile {
            detail: "nonce is not valid base64".into(),
        })?;
    let ciphertext = BASE64
        .decode(&sec.ciphertext)
        .map_err(|_| IngestError::KeyFile {
            detail: "ciphertext is not valid base64".into(),
        })?;
    if nonce_bytes.len() != SECRETBOX_NONCE_LEN {
        return Err(IngestError::KeyFile {
            detail: "nonce has the wrong length".into(),
        });
    }

    let mut derived = derive_passphrase_key(passphrase, &salt, &sec.argon2)?;
    let cipher = XSalsa20Poly1305::new_from_slice(&derived).map_err(|_| IngestError::Crypto)?;
    derived.zeroize();
    let nonce = Nonce::from_slice(&nonce_bytes);
    let mut plaintext = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|_| IngestError::Crypto)?;

    if plaintext.len() != SECRET_KEY_LEN {
        plaintext.zeroize();
        return Err(IngestError::Crypto);
    }
    let mut sk_bytes = [0u8; SECRET_KEY_LEN];
    sk_bytes.copy_from_slice(&plaintext);
    plaintext.zeroize();
    let secret = SecretKey(sk_bytes);
    sk_bytes.zeroize();

    // Bind the recovered key to the envelope's declared public half: catches a
    // corrupt envelope whose secretbox somehow opened to the wrong scalar.
    if secret.public_key() != public {
        return Err(IngestError::Crypto);
    }
    Ok(secret)
}

/// Serializes a `public` key-file envelope.
pub fn serialize_public_key(
    key: &PublicKey,
    metadata: KeyMetadata,
) -> Result<Vec<u8>, IngestError> {
    let env = KeyFileEnvelope {
        format: KEY_FILE_FORMAT.to_string(),
        schema_version: key_file_current_version(),
        role: KeyRole::Public,
        created_at: metadata.created_at,
        fingerprint: fingerprint(key).to_string(),
        public_key: encode_hex(key.as_bytes()),
        secret: None,
        extras: BTreeMap::new(),
    };
    serde_json::to_vec(&env).map_err(|e| IngestError::Serialize(e.to_string()))
}

/// Serializes a `secret-encrypted` key-file envelope: the secret key under an
/// Argon2id-derived XSalsa20-Poly1305 secretbox.
pub fn serialize_secret_key(
    key: &SecretKey,
    passphrase: &str,
    metadata: KeyMetadata,
) -> Result<Vec<u8>, IngestError> {
    let params = Argon2Params {
        m_cost: DEFAULT_ARGON2_M_COST,
        t_cost: DEFAULT_ARGON2_T_COST,
        p_cost: DEFAULT_ARGON2_P_COST,
    };
    let mut salt = [0u8; ARGON2_SALT_LEN];
    OsRng.fill_bytes(&mut salt);
    let mut nonce_bytes = [0u8; SECRETBOX_NONCE_LEN];
    OsRng.fill_bytes(&mut nonce_bytes);

    let mut derived = derive_passphrase_key(passphrase, &salt, &params)?;
    let cipher = XSalsa20Poly1305::new_from_slice(&derived).map_err(|_| IngestError::Crypto)?;
    derived.zeroize();
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, key.expose_bytes().as_ref())
        .map_err(|_| IngestError::Crypto)?;

    let public = key.public_key();
    let env = KeyFileEnvelope {
        format: KEY_FILE_FORMAT.to_string(),
        schema_version: key_file_current_version(),
        role: KeyRole::SecretEncrypted,
        created_at: metadata.created_at,
        fingerprint: fingerprint(&public).to_string(),
        public_key: encode_hex(public.as_bytes()),
        secret: Some(SecretEnvelope {
            kdf: KDF_ARGON2ID.to_string(),
            argon2: params,
            salt: BASE64.encode(salt),
            cipher: CIPHER_XSALSA20POLY1305.to_string(),
            nonce: BASE64.encode(nonce_bytes),
            ciphertext: BASE64.encode(ciphertext),
        }),
        extras: BTreeMap::new(),
    };
    serde_json::to_vec(&env).map_err(|e| IngestError::Serialize(e.to_string()))
}

/// Derives a 32-byte symmetric key from a passphrase with Argon2id and the
/// stored parameters. The returned array is the caller's to zeroize.
fn derive_passphrase_key(
    passphrase: &str,
    salt: &[u8],
    params: &Argon2Params,
) -> Result<[u8; 32], IngestError> {
    let params = Params::new(
        params.m_cost,
        params.t_cost,
        params.p_cost,
        Some(SECRET_KEY_LEN),
    )
    .map_err(|_| IngestError::KeyFile {
        detail: "invalid argon2 parameters".into(),
    })?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut derived = [0u8; 32];
    argon2
        .hash_password_into(passphrase.as_bytes(), salt, &mut derived)
        .map_err(|_| IngestError::Crypto)?;
    Ok(derived)
}

fn public_key_from_hex(hex: &str) -> Result<PublicKey, IngestError> {
    let bytes = decode_hex(hex)?;
    if bytes.len() != PUBLIC_KEY_LEN {
        return Err(IngestError::KeyFile {
            detail: "public key has the wrong length".into(),
        });
    }
    let mut pk = [0u8; PUBLIC_KEY_LEN];
    pk.copy_from_slice(&bytes);
    Ok(PublicKey(pk))
}

fn encode_hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        let _ = write!(out, "{b:02x}");
    }
    out
}

fn decode_hex(s: &str) -> Result<Vec<u8>, IngestError> {
    if !s.len().is_multiple_of(2) {
        return Err(IngestError::KeyFile {
            detail: "hex string has an odd length".into(),
        });
    }
    (0..s.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&s[i..i + 2], 16).map_err(|_| IngestError::KeyFile {
                detail: "invalid hex character".into(),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn meta() -> KeyMetadata {
        KeyMetadata {
            created_at: "2026-08-11T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn seal_open_round_trips() {
        let kp = generate_keypair();
        let msg = b"synthetic inner payload";
        let ct = seal(msg, &kp.public);
        assert_eq!(open(&ct, &kp).expect("open"), msg);
    }

    #[test]
    fn seal_is_anonymous_and_nondeterministic() {
        let kp = generate_keypair();
        let msg = b"same plaintext";
        let a = seal(msg, &kp.public);
        let b = seal(msg, &kp.public);
        // Fresh ephemeral key per seal -> different ciphertexts, both open.
        assert_ne!(a, b);
        assert_eq!(open(&a, &kp).unwrap(), msg);
        assert_eq!(open(&b, &kp).unwrap(), msg);
    }

    #[test]
    fn open_rejects_wrong_key() {
        let kp = generate_keypair();
        let other = generate_keypair();
        let ct = seal(b"secret", &kp.public);
        assert_eq!(open(&ct, &other), Err(IngestError::Crypto));
    }

    #[test]
    fn open_rejects_truncated_and_flipped() {
        let kp = generate_keypair();
        let ct = seal(b"secret", &kp.public);
        assert_eq!(open(&ct[..ct.len() - 1], &kp), Err(IngestError::Crypto));
        let mut flipped = ct.clone();
        let last = flipped.len() - 1;
        flipped[last] ^= 0x01;
        assert_eq!(open(&flipped, &kp), Err(IngestError::Crypto));
        // Flip inside the ephemeral public key prefix, too.
        let mut flipped_head = ct.clone();
        flipped_head[0] ^= 0x01;
        assert_eq!(open(&flipped_head, &kp), Err(IngestError::Crypto));
    }

    #[test]
    fn fingerprint_round_trips_through_string() {
        let kp = generate_keypair();
        let fp = fingerprint(&kp.public);
        let text = fp.to_string();
        // 8 groups of 4 hex, dash-separated.
        assert_eq!(text.len(), FINGERPRINT_LEN * 2 + (FINGERPRINT_LEN / 2 - 1));
        assert_eq!(text.split('-').count(), 8);
        assert!(text.split('-').all(|g| g.len() == 4));
        let parsed: Fingerprint = text.parse().expect("parse fingerprint");
        assert_eq!(parsed, fp);
    }

    #[test]
    fn fingerprint_from_str_rejects_malformed() {
        assert!("not-a-fingerprint".parse::<Fingerprint>().is_err());
        assert!("3f9a-1c02".parse::<Fingerprint>().is_err()); // too few groups
        assert!(
            "3f9a-1c02-77de-b410-8e55-a0c3-4d21-96fg"
                .parse::<Fingerprint>()
                .is_err()
        ); // non-hex 'g'
        assert!(
            "3f9-1c02-77de-b410-8e55-a0c3-4d21-96fb"
                .parse::<Fingerprint>()
                .is_err()
        ); // group too short
    }

    #[test]
    fn public_key_envelope_round_trips() {
        let kp = generate_keypair();
        let bytes = serialize_public_key(&kp.public, meta()).expect("serialize");
        let parsed = parse_public_key(&bytes).expect("parse");
        assert_eq!(parsed, kp.public);
    }

    #[test]
    fn secret_key_envelope_round_trips() {
        let kp = generate_keypair();
        let bytes = serialize_secret_key(&kp.secret, "correct horse battery staple river", meta())
            .expect("serialize");
        let parsed = parse_secret_key(&bytes, "correct horse battery staple river").expect("parse");
        assert_eq!(parsed.expose_bytes(), kp.secret.expose_bytes());
        // The recovered key still opens boxes sealed to the original public key.
        let ct = seal(b"through the envelope", &kp.public);
        let recovered = Keypair {
            public: kp.public,
            secret: parsed,
        };
        assert_eq!(open(&ct, &recovered).unwrap(), b"through the envelope");
    }

    #[test]
    fn secret_key_envelope_rejects_wrong_passphrase() {
        let kp = generate_keypair();
        let bytes = serialize_secret_key(&kp.secret, "the right passphrase here", meta())
            .expect("serialize");
        // SecretKey has no PartialEq (no non-constant-time compare of key
        // material), so match on the error arm rather than assert_eq.
        assert!(matches!(
            parse_secret_key(&bytes, "the WRONG passphrase here"),
            Err(IngestError::Crypto)
        ));
    }

    #[test]
    fn parse_rejects_role_mismatch() {
        let kp = generate_keypair();
        let public_bytes = serialize_public_key(&kp.public, meta()).unwrap();
        // A public envelope is not a secret one.
        assert!(matches!(
            parse_secret_key(&public_bytes, "whatever"),
            Err(IngestError::KeyFile { .. })
        ));
        let secret_bytes =
            serialize_secret_key(&kp.secret, "passphrase goes right here", meta()).unwrap();
        assert!(matches!(
            parse_public_key(&secret_bytes),
            Err(IngestError::KeyFile { .. })
        ));
    }

    #[test]
    fn parse_rejects_unknown_major_version() {
        let kp = generate_keypair();
        let bytes = serialize_public_key(&kp.public, meta()).unwrap();
        let mut env: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        env["schema_version"] = serde_json::json!("9.0.0");
        let bumped = serde_json::to_vec(&env).unwrap();
        assert!(matches!(
            parse_public_key(&bumped),
            Err(IngestError::UnknownMajorVersion { .. })
        ));
    }

    #[test]
    fn parse_rejects_tampered_fingerprint() {
        let kp = generate_keypair();
        let bytes = serialize_public_key(&kp.public, meta()).unwrap();
        let mut env: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        env["fingerprint"] = serde_json::json!("0000-0000-0000-0000-0000-0000-0000-0000");
        let tampered = serde_json::to_vec(&env).unwrap();
        assert!(matches!(
            parse_public_key(&tampered),
            Err(IngestError::KeyFile { .. })
        ));
    }
}
