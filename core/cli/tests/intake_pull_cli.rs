//! `cn intake pull` entrypoint battery (blueprint intake-relay.md section 7).
//! These drive the REAL `cn` binary to cover the CLI seam the injected-core
//! tests (intake_pull.rs) cannot reach: argument parsing, config load/version,
//! and the key-load + pin/passphrase cross-checks in `run`/`load_keypair`. Every
//! case fails BEFORE any network I/O (bad args, bad config, or a key that will
//! not load), so no relay is needed. All key material is generated fresh in
//! tempdirs and never leaves them (I1).

use std::io::Write;
use std::path::Path;
use std::process::{Command, Output, Stdio};

/// A synthetic 6-word ceremony passphrase (test-only; never operational).
const PASS: &str = "correct horse battery staple river delta";

fn cn(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_cn"))
        .args(args)
        .stdin(Stdio::null())
        .output()
        .expect("cn runs")
}

fn cn_stdin(args: &[&str], stdin_data: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_cn"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("cn spawns");
    let _ = child
        .stdin
        .take()
        .expect("child stdin")
        .write_all(stdin_data.as_bytes());
    child.wait_with_output().expect("cn completes")
}

fn code(output: &Output) -> i32 {
    output.status.code().expect("cn exits with a code")
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

/// Generates a keypair into `key_dir` and returns its fingerprint.
fn keygen(key_dir: &Path) -> String {
    let out = cn_stdin(
        &["intake", "keygen", "--out", key_dir.to_str().unwrap()],
        &format!("{PASS}\n{PASS}\n"),
    );
    assert_eq!(code(&out), 0, "keygen stderr: {}", stderr(&out));
    let report: serde_json::Value = serde_json::from_slice(&out.stdout).expect("keygen JSON");
    report["fingerprint"]
        .as_str()
        .expect("fingerprint")
        .to_string()
}

/// Writes a puller config JSON to `path`, pinning `fingerprint`, and returns the
/// path as a string. `version` lets a test force an unknown major.
fn write_config(
    dir: &Path,
    path: &Path,
    key_dir: &Path,
    fingerprint: &str,
    credential: &Path,
    version: &str,
) {
    let config = serde_json::json!({
        "config_version": version,
        "key_dir": key_dir.to_str().unwrap(),
        "key_fingerprint_pin": fingerprint,
        "relay_origin": "https://relay.example.workers.dev",
        "credential_path": credential.to_str().unwrap(),
        "pages_origin": "https://example.github.io/community-connector",
        "manifest_path": dir.join("dist.manifest.json").to_str().unwrap(),
        "manifest_pin": {
            "manifest_hash": "0".repeat(64),
            "commit_sha": "abc123",
            "pinned_at": "2026-08-11T00:00:00Z",
            "pinned_by": "test",
        },
        "known_consent_digests": [],
        "blob_ttl_seconds": 86400,
        "max_pull_interval_seconds": 3600,
    });
    std::fs::write(path, serde_json::to_vec_pretty(&config).unwrap()).unwrap();
}

/// A valid, lockable queue root (a fresh tempdir subdir).
fn queue_dir(dir: &Path) -> std::path::PathBuf {
    let queue = dir.join("queue");
    std::fs::create_dir_all(&queue).unwrap();
    queue
}

#[test]
fn missing_config_is_a_usage_error() {
    let dir = tempfile::tempdir().unwrap();
    let out = cn(&[
        "intake",
        "pull",
        "--queue",
        queue_dir(dir.path()).to_str().unwrap(),
    ]);
    assert_eq!(code(&out), 2, "missing --config is a usage error");
    assert!(stderr(&out).contains("--config is required"));
}

#[test]
fn missing_queue_is_a_usage_error() {
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("pull.json");
    std::fs::write(&config, b"{}").unwrap();
    let out = cn(&["intake", "pull", "--config", config.to_str().unwrap()]);
    assert_eq!(code(&out), 2, "missing --queue is a usage error");
    assert!(stderr(&out).contains("--queue is required"));
}

#[test]
fn unreadable_config_fails() {
    let dir = tempfile::tempdir().unwrap();
    let out = cn(&[
        "intake",
        "pull",
        "--config",
        dir.path().join("nope.json").to_str().unwrap(),
        "--queue",
        queue_dir(dir.path()).to_str().unwrap(),
    ]);
    assert_eq!(code(&out), 1);
    assert!(stderr(&out).contains("cannot read config"));
}

#[test]
fn unknown_config_major_version_fails() {
    let dir = tempfile::tempdir().unwrap();
    let key_dir = dir.path().join("keys");
    let fingerprint = keygen(&key_dir);
    let credential = dir.path().join("cred.txt");
    std::fs::write(&credential, b"test-token\n").unwrap();
    let config = dir.path().join("pull.json");
    write_config(
        dir.path(),
        &config,
        &key_dir,
        &fingerprint,
        &credential,
        "9.0.0",
    );

    let out = cn(&[
        "intake",
        "pull",
        "--config",
        config.to_str().unwrap(),
        "--queue",
        queue_dir(dir.path()).to_str().unwrap(),
    ]);
    assert_eq!(code(&out), 1);
    assert!(
        stderr(&out).contains("unknown puller config major version"),
        "stderr: {}",
        stderr(&out)
    );
}

#[test]
fn key_fingerprint_pin_mismatch_fails_before_network() {
    let dir = tempfile::tempdir().unwrap();
    let key_dir = dir.path().join("keys");
    keygen(&key_dir);
    let credential = dir.path().join("cred.txt");
    std::fs::write(&credential, b"test-token\n").unwrap();
    let config = dir.path().join("pull.json");
    // Pin a DIFFERENT fingerprint than the generated key.
    write_config(
        dir.path(),
        &config,
        &key_dir,
        "0000-0000-0000-0000-0000-0000-0000-0000",
        &credential,
        "0.1.0",
    );

    // Null stdin: the pin check reads public.json and fails BEFORE any
    // passphrase prompt, and long before any network call.
    let out = cn(&[
        "intake",
        "pull",
        "--config",
        config.to_str().unwrap(),
        "--queue",
        queue_dir(dir.path()).to_str().unwrap(),
    ]);
    assert_eq!(code(&out), 1);
    assert!(
        stderr(&out).contains("does not match the pinned fingerprint")
            || stderr(&out).contains("pins"),
        "stderr: {}",
        stderr(&out)
    );
}

#[test]
fn wrong_passphrase_fails_to_open_the_secret() {
    let dir = tempfile::tempdir().unwrap();
    let key_dir = dir.path().join("keys");
    let fingerprint = keygen(&key_dir);
    let credential = dir.path().join("cred.txt");
    std::fs::write(&credential, b"test-token\n").unwrap();
    let config = dir.path().join("pull.json");
    // Correct pin, so the flow reaches the passphrase-protected secret.
    write_config(
        dir.path(),
        &config,
        &key_dir,
        &fingerprint,
        &credential,
        "0.1.0",
    );

    let out = cn_stdin(
        &[
            "intake",
            "pull",
            "--config",
            config.to_str().unwrap(),
            "--queue",
            queue_dir(dir.path()).to_str().unwrap(),
        ],
        "the wrong passphrase entirely\n",
    );
    assert_eq!(code(&out), 1);
    assert!(
        stderr(&out).contains("cannot open secret key"),
        "stderr: {}",
        stderr(&out)
    );
    // The failure never echoes the passphrase.
    assert!(!stderr(&out).contains("the wrong passphrase entirely"));
}
