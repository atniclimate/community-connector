//! `cn intake pull` D8 bundle-verification battery (blueprint intake-relay.md
//! section 8 step 8; ADR-005 D8). `verify_bundle` takes an injected `fetch`
//! closure, so the whole decision table runs here with canned responses - no
//! real server, no network. All fixtures are synthetic and live in tempdirs;
//! nothing is committed (I1).

use std::collections::HashMap;
use std::path::PathBuf;

use cn::intake::bundle::{BundleResult, verify_bundle};
use cn::intake::pull::{HttpResult, PullConfig, parse_config};
use sha2::{Digest, Sha256};

const KEY_FP: &str = "3f9a-1c02-77de-b410-8e55-a0c3-4d21-96fb";
const OTHER_FP: &str = "0000-0000-0000-0000-0000-0000-0000-0000";
const PAGES: &str = "https://example.github.io/community-connector";

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

/// A synthetic deployment: the pinned manifest FILE on disk, a correctly-pinned
/// config (key_fingerprint_pin = KEY_FP), and the map of good deployed
/// responses keyed by full URL.
struct Deployment {
    _dir: tempfile::TempDir,
    manifest_path: PathBuf,
    config: PullConfig,
    good: HashMap<String, Vec<u8>>,
}

/// Builds a deployment whose manifest lists `files` and stamps `manifest_fp` as
/// the embedded key fingerprint (pass KEY_FP for a match, OTHER_FP to force the
/// step-4 mismatch).
fn deployment(files: &[(&str, &[u8])], manifest_fp: &str) -> Deployment {
    let dir = tempfile::tempdir().expect("tempdir");
    let entries: Vec<serde_json::Value> = files
        .iter()
        .map(|&(path, body)| {
            serde_json::json!({ "path": path, "bytes": body.len(), "sha256": sha256_hex(body) })
        })
        .collect();
    let manifest = serde_json::json!({
        "files": entries,
        "provenance": { "key_fingerprint": manifest_fp, "form_version": "0.1.0" },
    });
    let manifest_bytes = serde_json::to_vec(&manifest).expect("manifest bytes");
    let manifest_path = dir.path().join("dist.manifest.json");
    std::fs::write(&manifest_path, &manifest_bytes).expect("write manifest");
    let manifest_hash = sha256_hex(&manifest_bytes);

    let config = config_for(dir.path(), &manifest_path, &manifest_hash);

    let mut good = HashMap::new();
    for &(path, body) in files {
        good.insert(format!("{PAGES}/{path}"), body.to_vec());
    }
    Deployment {
        _dir: dir,
        manifest_path,
        config,
        good,
    }
}

fn config_for(
    dir: &std::path::Path,
    manifest_path: &std::path::Path,
    manifest_hash: &str,
) -> PullConfig {
    let json = serde_json::json!({
        "config_version": "0.1.0",
        "key_dir": dir.join("keys").to_str().unwrap(),
        "key_fingerprint_pin": KEY_FP,
        "relay_origin": "https://relay.example.workers.dev",
        "credential_path": dir.join("cred.txt").to_str().unwrap(),
        "pages_origin": PAGES,
        "manifest_path": manifest_path.to_str().unwrap(),
        "manifest_pin": {
            "manifest_hash": manifest_hash,
            "commit_sha": "abc123",
            "pinned_at": "2026-08-11T00:00:00Z",
            "pinned_by": "test",
        },
        "known_consent_digests": [],
        "blob_ttl_seconds": 86400,
        "max_pull_interval_seconds": 3600,
    });
    parse_config(&serde_json::to_vec(&json).unwrap()).expect("config parses")
}

/// A closure serving a static url->body map as 200s, else a transport error.
fn serve(map: HashMap<String, Vec<u8>>) -> impl Fn(&str) -> HttpResult {
    move |url: &str| {
        map.get(url)
            .cloned()
            .map(|body| (200u16, body))
            .ok_or_else(|| format!("no route for {url}"))
    }
}

// 1. All files match -> Verified with the right count + fingerprint.
#[test]
fn all_files_match_verifies() {
    let dep = deployment(
        &[
            ("index.html", b"<html>hi</html>"),
            ("assets/app.js", b"console.log(1)"),
        ],
        KEY_FP,
    );
    let result = verify_bundle(&dep.config, serve(dep.good.clone()));
    match result {
        BundleResult::Verified {
            files_checked,
            key_fingerprint,
        } => {
            assert_eq!(files_checked, 2);
            assert_eq!(key_fingerprint, KEY_FP);
        }
        other => panic!("expected Verified, got {other:?}"),
    }
}

// 2. One file's bytes differ -> Failed, naming the file.
#[test]
fn hash_mismatch_fails() {
    let dep = deployment(&[("index.html", b"<html>hi</html>")], KEY_FP);
    let mut served = dep.good.clone();
    served.insert(
        format!("{PAGES}/index.html"),
        b"<html>TAMPERED</html>".to_vec(),
    );
    match verify_bundle(&dep.config, serve(served)) {
        BundleResult::Failed { errors } => {
            assert!(
                errors
                    .iter()
                    .any(|e| e.contains("index.html") && e.contains("sha256"))
            );
        }
        other => panic!("expected Failed, got {other:?}"),
    }
}

// 3. One file's length differs -> Failed with a length complaint.
#[test]
fn length_mismatch_fails() {
    let dep = deployment(&[("index.html", b"<html>hi</html>")], KEY_FP);
    let mut served = dep.good.clone();
    // Truncated body: different length (and, necessarily, a different hash).
    served.insert(format!("{PAGES}/index.html"), b"<ht".to_vec());
    match verify_bundle(&dep.config, serve(served)) {
        BundleResult::Failed { errors } => {
            assert!(
                errors
                    .iter()
                    .any(|e| e.contains("index.html") && e.contains("length"))
            );
        }
        other => panic!("expected Failed, got {other:?}"),
    }
}

// 4. One file returns 404 -> Failed.
#[test]
fn not_found_fails() {
    let dep = deployment(&[("index.html", b"<html>hi</html>")], KEY_FP);
    let fetch = |_url: &str| Ok((404u16, Vec::new()));
    match verify_bundle(&dep.config, fetch) {
        BundleResult::Failed { errors } => {
            assert!(
                errors
                    .iter()
                    .any(|e| e.contains("index.html") && e.contains("404"))
            );
        }
        other => panic!("expected Failed, got {other:?}"),
    }
}

// 5. One file returns a 301 redirect -> Failed (no redirects allowed).
#[test]
fn redirect_fails() {
    let dep = deployment(&[("index.html", b"<html>hi</html>")], KEY_FP);
    let fetch = |_url: &str| Ok((301u16, Vec::new()));
    match verify_bundle(&dep.config, fetch) {
        BundleResult::Failed { errors } => {
            assert!(errors.iter().any(|e| e.contains("301")));
        }
        other => panic!("expected Failed, got {other:?}"),
    }
}

// 6. The pinned manifest file's hash no longer matches the pin -> Failed.
#[test]
fn tampered_manifest_fails() {
    let dep = deployment(&[("index.html", b"<html>hi</html>")], KEY_FP);
    // Rewrite the manifest FILE after pinning: its hash now diverges.
    std::fs::write(
        &dep.manifest_path,
        b"{\"files\":[],\"provenance\":{\"key_fingerprint\":\"x\"}}",
    )
    .expect("rewrite manifest");
    // The fetch must never be reached; a panic here would fail the test.
    let fetch = |_url: &str| -> HttpResult { panic!("fetch must not run when the pin fails") };
    match verify_bundle(&dep.config, fetch) {
        BundleResult::Failed { errors } => {
            assert!(errors.iter().any(|e| e.contains("manifest hash mismatch")));
        }
        other => panic!("expected Failed, got {other:?}"),
    }
}

// 7. The manifest's embedded key fingerprint diverges from the config pin -> Failed.
#[test]
fn key_fingerprint_mismatch_fails() {
    let dep = deployment(&[("index.html", b"<html>hi</html>")], OTHER_FP);
    let fetch =
        |_url: &str| -> HttpResult { panic!("fetch must not run when the fingerprint fails") };
    match verify_bundle(&dep.config, fetch) {
        BundleResult::Failed { errors } => {
            assert!(errors.iter().any(|e| e.contains("key fingerprint")));
        }
        other => panic!("expected Failed, got {other:?}"),
    }
}

// 8. The Pages origin is unreachable on the first fetch -> Degraded.
#[test]
fn unreachable_origin_degrades() {
    let dep = deployment(&[("index.html", b"<html>hi</html>")], KEY_FP);
    let fetch = |_url: &str| -> HttpResult { Err("connection refused".to_string()) };
    match verify_bundle(&dep.config, fetch) {
        BundleResult::Degraded { reason } => {
            assert!(reason.contains("unreachable"));
        }
        other => panic!("expected Degraded, got {other:?}"),
    }
}

// 9. The pinned manifest file is missing on disk -> Failed (cannot verify).
#[test]
fn missing_manifest_fails() {
    let dep = deployment(&[("index.html", b"<html>hi</html>")], KEY_FP);
    let mut config = dep.config.clone();
    config.manifest_path = dep._dir.path().join("does-not-exist.json");
    let fetch =
        |_url: &str| -> HttpResult { panic!("fetch must not run when the manifest is missing") };
    match verify_bundle(&config, fetch) {
        BundleResult::Failed { errors } => {
            assert!(
                errors
                    .iter()
                    .any(|e| e.contains("cannot read pinned manifest"))
            );
        }
        other => panic!("expected Failed, got {other:?}"),
    }
}
