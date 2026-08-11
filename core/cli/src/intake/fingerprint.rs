//! `cn intake fingerprint <public.json>`: parse a public-key file and print its
//! fingerprint (blueprint intake-relay.md section 3). Positional arg, matching
//! the blueprint's literal syntax. `parse_public_key` already rejects a
//! secret-encrypted file, a wrong-length key, an unknown major, or a tampered
//! fingerprint with a clear `KeyFile`/`UnknownMajorVersion` error (crypto.rs
//! tests); this surfaces that loudly (I3).
//!
//! The stdout report carries no timestamp, so repeated runs against the same
//! file are byte-identical (the fingerprint is a deterministic function of the
//! key) - the by-hand and by-eye verification the ceremony depends on.

use std::io::Write;

use serde_json::json;

use cn_ingest::{fingerprint, parse_public_key};

use crate::Exit;

pub(crate) fn run(
    args: &[String],
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> std::io::Result<Exit> {
    let path = match parse_args(args) {
        Ok(path) => path,
        Err(message) => {
            writeln!(err, "error: {message}")?;
            super::usage_to(err)?;
            return Ok(Exit::Usage);
        }
    };
    let bytes = match std::fs::read(&path) {
        Ok(bytes) => bytes,
        Err(io_err) => {
            writeln!(err, "error: cannot read '{path}': {io_err}")?;
            return Ok(Exit::Failure);
        }
    };
    let public = match parse_public_key(&bytes) {
        Ok(public) => public,
        Err(ingest_err) => {
            writeln!(
                err,
                "error: '{path}' is not a valid public key file: {ingest_err}"
            )?;
            return Ok(Exit::Failure);
        }
    };
    let report = json!({
        "command": "fingerprint",
        "path": path,
        "fingerprint": fingerprint(&public).to_string(),
    });
    super::emit_report(out, &report)?;
    Ok(Exit::Ok)
}

fn parse_args(args: &[String]) -> Result<String, String> {
    match args {
        [] => Err("fingerprint needs a <public.json> path".to_string()),
        [path] if !path.starts_with('-') => Ok(path.clone()),
        [flagish] => Err(format!("unknown intake fingerprint argument '{flagish}'")),
        _ => Err("fingerprint takes exactly one <public.json> path".to_string()),
    }
}
