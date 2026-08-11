//! `cn intake`: the native durable owner of the intake queue (ADR-005 D4;
//! docs/blueprints/intake-pipeline.md sections 1-2 and 9 step 5) plus the
//! offline keygen-ceremony tooling for the remote relay (ADR-005 D3;
//! docs/blueprints/intake-relay.md section 3; docs/design/
//! facilitator-keygen-ceremony.md).
//!
//! The CLI executes I/O for the pure cn-ingest verdicts and crypto; every trust
//! decision - admission, planning, authorization, fold acceptance, and the
//! sealed-box/key-file logic - lives in the core crates (I2). This module is
//! argument marshalling and dispatch only. The keygen family is offline by
//! construction (ceremony section 3 point 4); the ONE component that crosses the
//! network boundary is `pull` (the remote puller, blueprint section 6), whose
//! HTTP client stays scoped to this CLI crate and never enters a cn-* crate
//! (ADR-005 D1 module fence). The D8 bundle verification `pull` depends on lives
//! in `bundle`, also CLI-only for the same reason.

mod apply;
mod backup;
pub mod bundle;
mod fingerprint;
mod keygen;
mod keymat;
pub mod pull;
mod queue;
mod selftest;

use std::io::Write;

use crate::Exit;

const USAGE: &str = "usage: cn intake <subcommand> [arguments]

Subcommands:
  apply --queue <queue-root> --ops <ops.jsonl> --group <group-uuid>
        --facilitator <person-uuid> [--kind <kind-id>]
      Run the intake apply transaction as the queue's single native mutator
      (ADR-005 D4): startup recovery, decision-inbox admission, approval
      planning, the idempotent durable append, completion transaction events,
      and the machine-readable run report (JSON on stdout, I12). The queue
      root must lie OUTSIDE any git worktree or cloud-synced directory; a
      single-instance lock serializes native mutators. --kind is the default
      entity kind for approvals whose payload carries no `kind` field.

  keygen --out <dir>
      Generate an X25519 intake keypair OFFLINE. Prompts for a >= 6-word
      passphrase (with confirmation), writes create-only public.json and
      secret.json (secret passphrase-encrypted) to <dir>, prints the
      fingerprint banner and the plaintext printed-backup block to stderr
      (never to any file), and an I12 metadata report to stdout.

  fingerprint <public.json>
      Print the fingerprint of a public key file (deterministic; no clock).

  selftest (--dry | --key-dir <dir>)
      Prove the sealed-box round trip. --dry uses an ephemeral in-memory key
      and touches no filesystem (ceremony PREP check). --key-dir loads the
      real pair, prompts for the passphrase, cross-checks the halves, and
      runs the round trip.

  backup verify [--from-print] <path>
      Prove a key backup opens. Default: <path> is a secret-encrypted
      envelope (prompts for the passphrase). --from-print: <path> is a text
      file holding the printed-backup base32 + check line (no passphrase).
      Either way it derives the public half, prints the fingerprint, and runs
      the round trip.

  pull --config <path> --queue <queue-root>
      Pull remote submissions from the relay (blueprint section 6; ADR-005
      D1/D3/D4/D6). Verifies the pinned deploy bundle (D8), then for each
      sealed blob: transport-dedup, fetch, decrypt, consent-check,
      semantic-dedup, stage a QueueRecord with SubmissionSource::Remote into
      <queue-root> (the same queue `intake apply` owns), and delete the relay
      blob only after verified staging. Prompts for the key passphrase; emits
      an I12 run report (JSON on stdout). This is the ONLY networked
      subcommand; the HTTP client is CLI-scoped (D1 module fence).

The keygen family makes no network calls and writes no secret material to any
tool-created file (ceremony design sections 3-5); `pull` is the sole networked
subcommand.";

pub(crate) fn run(
    args: &[String],
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> std::io::Result<Exit> {
    match args.first().map(String::as_str) {
        Some("apply") => apply::run(&args[1..], out, err),
        Some("keygen") => keygen::run(&args[1..], out, err),
        Some("fingerprint") => fingerprint::run(&args[1..], out, err),
        Some("selftest") => selftest::run(&args[1..], out, err),
        Some("backup") => backup::run(&args[1..], out, err),
        Some("pull") => pull::run(&args[1..], out, err),
        Some(other) => {
            writeln!(err, "error: unknown intake subcommand '{other}'")?;
            writeln!(err, "{USAGE}")?;
            Ok(Exit::Usage)
        }
        None => {
            writeln!(err, "error: missing intake subcommand")?;
            writeln!(err, "{USAGE}")?;
            Ok(Exit::Usage)
        }
    }
}

pub(crate) fn usage_to(err: &mut dyn Write) -> std::io::Result<()> {
    writeln!(err, "{USAGE}")
}

/// Renders an I12 report as pretty JSON on stdout. A serialization failure is
/// surfaced as a JSON object rather than swallowed (I3), matching apply.rs.
pub(crate) fn emit_report(out: &mut dyn Write, report: &serde_json::Value) -> std::io::Result<()> {
    let rendered = serde_json::to_string_pretty(report)
        .unwrap_or_else(|render_err| format!("{{\"render_error\":\"{render_err}\"}}"));
    writeln!(out, "{rendered}")
}
