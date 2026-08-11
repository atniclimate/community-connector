//! `cn intake backup verify [--from-print] <path>`: prove a key backup opens
//! (blueprint intake-relay.md section 3; ceremony section 5 restore drill,
//! checklist steps 11 and 15). A small nested router under `backup` expects
//! `verify`, mirroring how intake/mod.rs itself dispatches `apply`.
//!
//! One backup artifact plus (in envelope mode) its passphrase is sufficient to
//! verify end to end: once the secret key is recovered the public half is
//! DERIVED via `SecretKey::public_key()`, so no paired public.json is needed
//! (the blueprint says "decrypt A key file", singular). `--from-print` is
//! passphrase-free by design: the printed backup is plaintext under physical
//! security, so making it depend on the passphrase would turn the passphrase
//! into a second single point of total loss - exactly what the two-backup
//! design avoids (ceremony section 5).

use std::io::Write;

use serde_json::json;

use cn_ingest::{Keypair, fingerprint, parse_secret_key};

use super::keymat;
use crate::Exit;

pub(crate) fn run(
    args: &[String],
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> std::io::Result<Exit> {
    match args.first().map(String::as_str) {
        Some("verify") => verify(&args[1..], out, err),
        Some(other) => {
            writeln!(err, "error: unknown intake backup subcommand '{other}'")?;
            super::usage_to(err)?;
            Ok(Exit::Usage)
        }
        None => {
            writeln!(
                err,
                "error: missing intake backup subcommand (expected 'verify')"
            )?;
            super::usage_to(err)?;
            Ok(Exit::Usage)
        }
    }
}

struct VerifyArgs {
    from_print: bool,
    path: String,
}

fn verify(args: &[String], out: &mut dyn Write, err: &mut dyn Write) -> std::io::Result<Exit> {
    let parsed = match parse_verify_args(args) {
        Ok(parsed) => parsed,
        Err(message) => {
            writeln!(err, "error: {message}")?;
            super::usage_to(err)?;
            return Ok(Exit::Usage);
        }
    };
    match run_verify(&parsed, err) {
        Ok((report, fp)) => {
            super::emit_report(out, &report)?;
            write_fingerprint_line(err, &fp)?;
            Ok(Exit::Ok)
        }
        Err(message) => {
            writeln!(err, "error: {message}")?;
            Ok(Exit::Failure)
        }
    }
}

fn parse_verify_args(args: &[String]) -> Result<VerifyArgs, String> {
    let mut from_print = false;
    let mut path = None;
    for arg in args {
        match arg.as_str() {
            "--from-print" => {
                if from_print {
                    return Err("--from-print given more than once".to_string());
                }
                from_print = true;
            }
            other if other.starts_with("--") => {
                return Err(format!("unknown intake backup verify argument '{other}'"));
            }
            other => {
                if path.replace(other.to_string()).is_some() {
                    return Err("backup verify takes exactly one <path>".to_string());
                }
            }
        }
    }
    Ok(VerifyArgs {
        from_print,
        path: path.ok_or_else(|| "backup verify needs a <path>".to_string())?,
    })
}

fn run_verify(
    args: &VerifyArgs,
    err: &mut dyn Write,
) -> Result<(serde_json::Value, String), String> {
    let secret = if args.from_print {
        let contents = std::fs::read_to_string(&args.path).map_err(|io_err| {
            format!("cannot read printed-backup file '{}': {io_err}", args.path)
        })?;
        keymat::parse_print_backup_file(&contents)?
    } else {
        let bytes = std::fs::read(&args.path)
            .map_err(|io_err| format!("cannot read secret key file '{}': {io_err}", args.path))?;
        let passphrase = keymat::read_existing_passphrase(err, "open the secret key backup")?;
        parse_secret_key(&bytes, &passphrase)
            .map_err(|ingest_err| format!("cannot open '{}': {ingest_err}", args.path))?
    };
    // Derive the public half from the recovered secret and prove the real
    // decrypt path against it.
    let public = secret.public_key();
    let fp = fingerprint(&public).to_string();
    let keypair = Keypair { public, secret };
    keymat::round_trip(&keypair)
        .map_err(|ingest_err| format!("backup verify round trip failed: {ingest_err}"))?;
    let mode = if args.from_print { "print" } else { "envelope" };
    let report = json!({
        "command": "backup-verify",
        "mode": mode,
        "path": args.path,
        "fingerprint": fp,
        "result": "passed",
    });
    Ok((report, fp))
}

fn write_fingerprint_line(err: &mut dyn Write, fingerprint: &str) -> std::io::Result<()> {
    writeln!(err)?;
    writeln!(
        err,
        "backup verified - recovered key fingerprint (compare by eye to the ceremony \
         sheet, steps 11/15):"
    )?;
    writeln!(err, "      {fingerprint}")
}
