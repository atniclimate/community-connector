//! `cn intake keygen --out <dir>`: the offline key-generation ceremony action
//! (blueprint intake-relay.md section 3; ceremony design sections 4-5,
//! checklist steps 7-10).
//!
//! Create-only: it refuses if either key file already exists, and writes with
//! `create_new` so a race cannot overwrite one either (simpler than queue.rs's
//! temp-then-durable-rename dance on purpose - keygen is a one-shot ceremony
//! action with no concurrent-writer or crash-recovery story). The I12 JSON
//! report on stdout carries ONLY non-secret metadata, since stdout is the
//! stream most likely to be redirected into a log. The fingerprint banner and
//! the plaintext printed-backup block go to stderr and are NEVER written to any
//! file the tool creates - the printed backup exists only on paper under
//! physical security, so the passphrase and the print backup never share a
//! failure mode (ceremony section 5). Auto-saving it anywhere would defeat that.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use serde_json::json;

use cn_ingest::{fingerprint, generate_keypair, serialize_public_key, serialize_secret_key};

use super::keymat;
use crate::Exit;

/// Everything the streams need after the fs/crypto work succeeds. The secret
/// key itself never escapes `run_keygen`; only its printed-backup encoding
/// (which must be printed) leaves as strings.
struct KeygenOutcome {
    report: serde_json::Value,
    fingerprint: String,
    print_base32: String,
    print_check: String,
}

pub(crate) fn run(
    args: &[String],
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> std::io::Result<Exit> {
    let out_dir = match parse_args(args) {
        Ok(dir) => dir,
        Err(message) => {
            writeln!(err, "error: {message}")?;
            super::usage_to(err)?;
            return Ok(Exit::Usage);
        }
    };
    let outcome = match run_keygen(&out_dir, err) {
        Ok(outcome) => outcome,
        Err(message) => {
            writeln!(err, "error: {message}")?;
            return Ok(Exit::Failure);
        }
    };
    super::emit_report(out, &outcome.report)?;
    write_fingerprint_banner(err, &outcome.fingerprint)?;
    write_print_backup_block(err, &outcome.print_base32, &outcome.print_check)?;
    Ok(Exit::Ok)
}

fn parse_args(args: &[String]) -> Result<String, String> {
    let mut out = None;
    let mut iter = args.iter();
    while let Some(flag) = iter.next() {
        match flag.as_str() {
            "--out" => {
                let value = iter
                    .next()
                    .ok_or_else(|| "--out needs a value".to_string())?;
                if out.replace(value.clone()).is_some() {
                    return Err("--out given more than once".to_string());
                }
            }
            other => return Err(format!("unknown intake keygen argument '{other}'")),
        }
    }
    out.ok_or_else(|| "--out is required".to_string())
}

fn run_keygen(out_dir: &str, err: &mut dyn Write) -> Result<KeygenOutcome, String> {
    let dir = Path::new(out_dir);
    fs::create_dir_all(dir)
        .map_err(|io_err| format!("cannot create output directory '{out_dir}': {io_err}"))?;
    let public_path = dir.join("public.json");
    let secret_path = dir.join("secret.json");

    // Create-only refusal BEFORE prompting for anything: a doomed keygen must
    // not even ask for a passphrase, and it writes nothing.
    for path in [&public_path, &secret_path] {
        if path_present(path)? {
            return Err(format!(
                "'{}' already exists; keygen is create-only and refuses to overwrite an \
                 existing key (nothing was written)",
                path.display()
            ));
        }
    }

    let passphrase = keymat::read_new_passphrase(err)?;
    let metadata = keymat::key_metadata_now()?;
    let keypair = generate_keypair();
    let fp = fingerprint(&keypair.public).to_string();

    let public_bytes = serialize_public_key(&keypair.public, metadata.clone())
        .map_err(|ingest_err| format!("cannot serialize public key: {ingest_err}"))?;
    let secret_bytes = serialize_secret_key(&keypair.secret, &passphrase, metadata.clone())
        .map_err(|ingest_err| format!("cannot serialize secret key: {ingest_err}"))?;

    // Write create-only. If the second write fails, best-effort-remove the
    // first so a retry stays create-only (create_new would otherwise refuse a
    // half-written pair). The public half is non-secret, so a stray one is not
    // a disclosure - only a usability wart, which the cleanup removes.
    create_new_file(&public_path, &public_bytes)?;
    if let Err(message) = create_new_file(&secret_path, &secret_bytes) {
        let _ = fs::remove_file(&public_path);
        return Err(format!(
            "{message}; removed the just-written '{}' so a retry stays create-only",
            public_path.display()
        ));
    }

    let (print_base32, print_check) = keymat::encode_print_backup(&keypair.secret);
    let report = json!({
        "command": "keygen",
        "fingerprint": fp,
        "public_key_path": public_path.display().to_string(),
        "secret_key_path": secret_path.display().to_string(),
        "created_at": metadata.created_at,
    });
    Ok(KeygenOutcome {
        report,
        fingerprint: fp,
        print_base32,
        print_check,
    })
}

/// Fail-closed existence probe: only `NotFound` counts as absent; any other
/// stat error refuses rather than assuming the path is free (I3).
fn path_present(path: &Path) -> Result<bool, String> {
    match fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(io_err) if io_err.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(io_err) => Err(format!(
            "cannot check whether '{}' exists ({io_err}); refusing to run blind (I3)",
            path.display()
        )),
    }
}

fn create_new_file(path: &PathBuf, bytes: &[u8]) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|io_err| format!("cannot create '{}': {io_err}", path.display()))?;
    file.write_all(bytes)
        .and_then(|()| file.flush())
        .and_then(|()| file.sync_all())
        .map_err(|io_err| format!("cannot write '{}': {io_err}", path.display()))
}

fn write_fingerprint_banner(err: &mut dyn Write, fingerprint: &str) -> std::io::Result<()> {
    let rule = "=".repeat(70);
    writeln!(err)?;
    writeln!(err, "{rule}")?;
    writeln!(
        err,
        "  INTAKE KEY FINGERPRINT - write all 8 groups by hand (ceremony step 8):"
    )?;
    writeln!(err)?;
    writeln!(err, "      {fingerprint}")?;
    writeln!(err)?;
    writeln!(err, "{rule}")
}

fn write_print_backup_block(err: &mut dyn Write, base32: &str, check: &str) -> std::io::Result<()> {
    writeln!(err)?;
    writeln!(
        err,
        "--- PRINTED BACKUP - transcribe onto the physical sheet (ceremony step 10) ---"
    )?;
    writeln!(err, "{}{}", keymat::PRINT_BACKUP_BASE32_PREFIX, base32)?;
    writeln!(err, "{}{}", keymat::PRINT_BACKUP_CHECK_PREFIX, check)?;
    writeln!(
        err,
        "WARNING: the two lines above ARE the plaintext secret key. This tool writes"
    )?;
    writeln!(
        err,
        "them to NO file. Transcribe them onto the physical backup sheet, then clear"
    )?;
    writeln!(
        err,
        "this terminal's scrollback. Anyone who reads them can decrypt every submission."
    )?;
    writeln!(
        err,
        "-----------------------------------------------------------------------------"
    )
}
