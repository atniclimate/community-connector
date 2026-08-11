//! `cn intake selftest`: prove the crypto stack round-trips (blueprint
//! intake-relay.md section 3; ceremony checklist steps 2 and 9). Exactly one of
//! `--dry` or `--key-dir <dir>` is required; both or neither is a usage error.
//!
//! - `--dry`: no filesystem access at all. Generate an ephemeral in-memory
//!   keypair and run the shared round-trip helper. This is the ceremony PREP
//!   check (step 2) that the crypto stack links and runs on this machine/build
//!   before any real key material exists - so it must not need `--key-dir`.
//! - `--key-dir <dir>`: load `public.json`, prompt for the passphrase, load
//!   `secret.json`, cross-check that the loaded secret's public half equals the
//!   loaded public key (so a mismatched pair fails loudly instead of silently
//!   testing the wrong pairing), then run the shared round trip.

use std::io::Write;
use std::path::Path;

use serde_json::json;

use cn_ingest::{Keypair, fingerprint, parse_public_key, parse_secret_key};

use super::keymat;
use crate::Exit;

enum Mode {
    Dry,
    KeyDir(String),
}

pub(crate) fn run(
    args: &[String],
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> std::io::Result<Exit> {
    let mode = match parse_args(args) {
        Ok(mode) => mode,
        Err(message) => {
            writeln!(err, "error: {message}")?;
            super::usage_to(err)?;
            return Ok(Exit::Usage);
        }
    };
    match run_selftest(mode, err) {
        Ok(report) => {
            super::emit_report(out, &report)?;
            Ok(Exit::Ok)
        }
        Err(message) => {
            writeln!(err, "error: {message}")?;
            Ok(Exit::Failure)
        }
    }
}

fn parse_args(args: &[String]) -> Result<Mode, String> {
    let mut dry = false;
    let mut key_dir = None;
    let mut iter = args.iter();
    while let Some(flag) = iter.next() {
        match flag.as_str() {
            "--dry" => {
                if dry {
                    return Err("--dry given more than once".to_string());
                }
                dry = true;
            }
            "--key-dir" => {
                let value = iter
                    .next()
                    .ok_or_else(|| "--key-dir needs a value".to_string())?;
                if key_dir.replace(value.clone()).is_some() {
                    return Err("--key-dir given more than once".to_string());
                }
            }
            other => return Err(format!("unknown intake selftest argument '{other}'")),
        }
    }
    match (dry, key_dir) {
        (true, None) => Ok(Mode::Dry),
        (false, Some(dir)) => Ok(Mode::KeyDir(dir)),
        (true, Some(_)) => Err("pass exactly one of --dry or --key-dir, not both".to_string()),
        (false, None) => Err("pass exactly one of --dry or --key-dir".to_string()),
    }
}

fn run_selftest(mode: Mode, err: &mut dyn Write) -> Result<serde_json::Value, String> {
    match mode {
        Mode::Dry => {
            let keypair = keymat::ephemeral_keypair();
            keymat::round_trip(&keypair)
                .map_err(|ingest_err| format!("dry selftest round trip failed: {ingest_err}"))?;
            Ok(json!({ "command": "selftest", "mode": "dry", "result": "passed" }))
        }
        Mode::KeyDir(dir) => run_key_dir_selftest(Path::new(&dir), err),
    }
}

fn run_key_dir_selftest(dir: &Path, err: &mut dyn Write) -> Result<serde_json::Value, String> {
    let public_path = dir.join("public.json");
    let public_bytes = std::fs::read(&public_path)
        .map_err(|io_err| format!("cannot read '{}': {io_err}", public_path.display()))?;
    let public = parse_public_key(&public_bytes).map_err(|ingest_err| {
        format!(
            "'{}' is not a valid public key file: {ingest_err}",
            public_path.display()
        )
    })?;

    let passphrase = keymat::read_existing_passphrase(err, "open the secret key")?;
    let secret_path = dir.join("secret.json");
    let secret_bytes = std::fs::read(&secret_path)
        .map_err(|io_err| format!("cannot read '{}': {io_err}", secret_path.display()))?;
    let secret = parse_secret_key(&secret_bytes, &passphrase)
        .map_err(|ingest_err| format!("cannot open '{}': {ingest_err}", secret_path.display()))?;

    // Cross-check the pair: a --key-dir pointed at a mismatched public/secret
    // must fail loudly, not silently certify the wrong pairing (I3).
    if secret.public_key() != public {
        return Err(
            "public.json and secret.json are not a matching pair (the secret key's public \
             half differs); refusing to certify a mismatched key directory (I3)"
                .to_string(),
        );
    }
    let fp = fingerprint(&public).to_string();
    let keypair = Keypair { public, secret };
    keymat::round_trip(&keypair)
        .map_err(|ingest_err| format!("key-dir selftest round trip failed: {ingest_err}"))?;
    Ok(json!({
        "command": "selftest",
        "mode": "key-dir",
        "key_dir": dir.display().to_string(),
        "fingerprint": fp,
        "result": "passed",
    }))
}
