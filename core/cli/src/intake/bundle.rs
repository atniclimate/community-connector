//! D8 deployed-bundle verification (blueprint docs/blueprints/intake-relay.md
//! section 6.1 precondition 4 and section 8 step 8; ADR-005 D8). Before the
//! puller decrypts anything, it proves the DEPLOYED Pages form still matches the
//! ceremony-pinned manifest: the exact files, byte-for-byte, built from the
//! exact key it holds. A tampered or replaced form must never solicit fresh
//! sealed submissions.
//!
//! This module needs HTTP, so it lives in the CLI crate, NOT cn-ingest (ADR-005
//! D1 module fence): the manifest fetch-and-compare is the second thing (after
//! the puller itself) that crosses the network boundary. It is written against
//! an injected `fetch` closure so the whole decision table is exercised in tests
//! with canned responses - no real server, no network in `cargo test`.
//!
//! The manifest CONTENT (the D8 file list) is read from a local, ceremony-pinned
//! file (`config.manifest_path`); the manifest is NOT deployed (D8 no-deploy
//! rule), so the deployed origin cannot serve it. The pin (`manifest_hash`)
//! guards that local file against substitution; the file's `provenance.
//! key_fingerprint` guards that the form was built for the key the puller holds.

use std::fs;

use serde::Deserialize;
use sha2::{Digest as _, Sha256};

use super::pull::{HttpResult, PullConfig};

/// The outcome of verifying the deployed bundle against the pinned manifest.
/// Serialized into the puller's precondition report (I12) via `#[serde(tag)]`,
/// so `bundle_check` reads `{"status":"verified",...}` etc.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum BundleResult {
    /// Every pinned file fetched, hashed, and length-matched. The puller may
    /// proceed and (post-pilot) solicit fresh submissions.
    Verified {
        files_checked: usize,
        key_fingerprint: String,
    },
    /// The Pages origin was unreachable on the FIRST fetch (connection refused,
    /// DNS failure, timeout). The puller proceeds on the key-pin precondition
    /// alone (the local key was already verified) but must NOT solicit new QR
    /// submissions until a future run verifies the bundle.
    Degraded { reason: String },
    /// A concrete mismatch: the pinned manifest failed its own hash, the
    /// embedded key fingerprint diverged, or a deployed file's bytes/length/
    /// status did not match. The bundle is compromised or misdeployed; the
    /// puller halts and touches nothing.
    Failed { errors: Vec<String> },
}

/// One `files[]` entry of the D8 manifest (`form/scripts/manifest.ts`
/// `ManifestFileEntry`): a deploy-root-relative path, byte length, and SHA-256.
/// Extra fields are ignored (forward compatibility).
#[derive(Debug, Deserialize)]
struct ManifestEntry {
    path: String,
    bytes: u64,
    sha256: String,
}

/// The D8 manifest provenance block - only `key_fingerprint` is consulted here
/// (the rest is human-facing provenance). Unknown fields are ignored.
#[derive(Debug, Deserialize)]
struct Provenance {
    key_fingerprint: String,
}

/// The pinned D8 manifest (`form/scripts/gen-manifest.mjs` output). Only the
/// file list and the embedded key fingerprint drive verification; unknown
/// top-level fields are ignored.
#[derive(Debug, Deserialize)]
struct Manifest {
    files: Vec<ManifestEntry>,
    provenance: Provenance,
}

/// Verifies the deployed Pages bundle against the ceremony-pinned manifest
/// (blueprint 6.1 precondition 4). `fetch` takes a full URL and returns
/// `(status, body_bytes)` or a transport-error string, so production passes a
/// real ureq caller and tests pass canned responses.
///
/// The order is deliberate: local integrity of the pin (manifest-hash, embedded
/// fingerprint) is proven BEFORE any network fetch, so a tampered local pin
/// fails closed without ever touching the origin. Only then are the deployed
/// files fetched and compared.
pub fn verify_bundle(config: &PullConfig, fetch: impl Fn(&str) -> HttpResult) -> BundleResult {
    // Step 1-2: read the pinned manifest FILE and prove it against its hash pin
    // before trusting a single byte of its content.
    let manifest_bytes = match fs::read(&config.manifest_path) {
        Ok(bytes) => bytes,
        Err(io_err) => {
            return BundleResult::Failed {
                errors: vec![format!(
                    "cannot read pinned manifest '{}': {io_err}",
                    config.manifest_path.display()
                )],
            };
        }
    };
    let manifest_hash = sha256_hex(&manifest_bytes);
    if !eq_ignore_ascii_case(&manifest_hash, &config.manifest_pin.manifest_hash) {
        return BundleResult::Failed {
            errors: vec![format!(
                "pinned manifest hash mismatch: file hashes to {manifest_hash}, pin expects {}",
                config.manifest_pin.manifest_hash
            )],
        };
    }

    // Step 3: parse the (now hash-verified) manifest.
    let manifest: Manifest = match serde_json::from_slice(&manifest_bytes) {
        Ok(manifest) => manifest,
        Err(parse_err) => {
            return BundleResult::Failed {
                errors: vec![format!("pinned manifest does not parse: {parse_err}")],
            };
        }
    };

    // Step 4: the manifest must have been built for the key the puller holds.
    if manifest.provenance.key_fingerprint != config.key_fingerprint_pin {
        return BundleResult::Failed {
            errors: vec![format!(
                "manifest key fingerprint '{}' does not match the pinned key '{}' - the form was \
                 built for a different key",
                manifest.provenance.key_fingerprint, config.key_fingerprint_pin
            )],
        };
    }

    // Step 5-7: fetch and compare every deployed file. A transport error on the
    // FIRST file means the origin never answered -> Degraded (proceed key-pin
    // only). A transport error AFTER a successful fetch, or any non-200 / hash /
    // length mismatch, is a concrete Failed.
    let base = config.pages_origin.trim_end_matches('/');
    let mut errors: Vec<String> = Vec::new();
    let mut any_fetched = false;
    for entry in &manifest.files {
        let url = format!("{base}/{}", entry.path);
        let (status, body) = match fetch(&url) {
            Ok(response) => response,
            Err(transport_err) => {
                if !any_fetched {
                    return BundleResult::Degraded {
                        reason: format!(
                            "Pages origin unreachable fetching '{url}' ({transport_err}); \
                             proceeding on key-pin only - no fresh QR solicitation until the \
                             bundle verifies on a later run"
                        ),
                    };
                }
                errors.push(format!(
                    "'{}': transport error after the origin had answered ({transport_err}) - \
                     a mid-verify origin fault is a deployment anomaly, not a clean unreachable",
                    entry.path
                ));
                continue;
            }
        };
        any_fetched = true;
        if status != 200 {
            errors.push(format!(
                "'{}': status {status} (expected 200; redirects and errors are deployment \
                 anomalies)",
                entry.path
            ));
            continue;
        }
        let actual_len = body.len() as u64;
        if actual_len != entry.bytes {
            errors.push(format!(
                "'{}': length {actual_len} != manifest {}",
                entry.path, entry.bytes
            ));
        }
        let actual_hash = sha256_hex(&body);
        if !eq_ignore_ascii_case(&actual_hash, &entry.sha256) {
            errors.push(format!(
                "'{}': sha256 {actual_hash} != manifest {}",
                entry.path, entry.sha256
            ));
        }
    }

    if errors.is_empty() {
        BundleResult::Verified {
            files_checked: manifest.files.len(),
            key_fingerprint: manifest.provenance.key_fingerprint,
        }
    } else {
        BundleResult::Failed { errors }
    }
}

/// SHA-256 (lowercase hex) over exact bytes - the D8 digest domain
/// (`form/scripts/manifest.ts` `sha256Hex`).
fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

/// Case-insensitive hex-digest comparison. Both sides are lowercase hex by
/// construction; folding case defends against a pin transcribed in uppercase
/// rather than silently failing a byte-identical bundle.
fn eq_ignore_ascii_case(a: &str, b: &str) -> bool {
    a.eq_ignore_ascii_case(b)
}
