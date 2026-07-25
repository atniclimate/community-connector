//! cn: Community Navigator CLI - zero-semantics command router (D-044.2).
//!
//! The router dispatches to thin adapters over already-ratified core
//! behavior: `validate` wraps `cn_schema::parse_template` and prints its
//! machine-readable validation report (I12); `export` wraps the existing
//! `cn_api::Api` load + export-snapshot path for a fixture ops log and one
//! viewer scope; `intake apply` is the native durable owner of the intake
//! queue (ADR-005 D4) - it executes I/O for cn-ingest's pure verdicts and
//! holds no trust semantics of its own (I2). `ingest` and `snapshot` are
//! parked stubs: ingest names the G-RAT human gate, snapshot names the
//! Phase 2 snapshot-envelope dependency. No routing semantics, no column
//! mapping, and no new schemas live in this crate (D-041, D-044.2).

mod export;
mod intake;
mod validate;

use std::io::Write;

/// Typed process exit codes for the cn router (D-044.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Exit {
    /// Successful run.
    Ok = 0,
    /// Operation attempted and failed (IO, validation errors, export failure).
    Failure = 1,
    /// Usage error or deliberately unavailable subcommand.
    Usage = 2,
}

impl From<Exit> for std::process::ExitCode {
    fn from(exit: Exit) -> Self {
        std::process::ExitCode::from(exit as u8)
    }
}

const USAGE: &str = "cn - Community Navigator CLI (reduced scope per DECISIONS.md D-044.2)

Usage:
  cn <command> [arguments]

Commands:
  validate <template.json>
      Validate a group template with cn-schema and print the
      machine-readable validation report as JSON. Exit 0 when the report
      has no errors; warnings are printed, never dropped.
  export --template <template.json> --ops <ops.jsonl>
         --group <group-uuid> --viewer <anonymous|person:<person-uuid>>
      Load a fixture ops log through cn-api and print the export-snapshot
      envelope for the given viewer scope.
  intake apply --queue <queue-root> --ops <ops.jsonl>
               --group <group-uuid> --facilitator <person-uuid> [--kind <kind-id>]
      Run the native intake apply transaction (ADR-005 D4): startup
      recovery, decision-inbox admission, approval planning, the
      idempotent durable append, and the I12 run report as JSON.
  ingest
      Not available: parked behind the G-RAT human gate.
  snapshot
      Not available: depends on the Phase 2 snapshot data envelope.
  help, --help, -h
      Print this help.

Exit codes:
  0  success
  1  operation failed (IO error, validation errors, load/export failure)
  2  usage error or unavailable subcommand";

/// Runs the CLI against `args` (program name excluded), writing to the
/// given streams.
///
/// Returns the typed exit code; propagates stream write failures to the
/// caller instead of swallowing them (I3).
pub fn run(args: &[String], out: &mut dyn Write, err: &mut dyn Write) -> std::io::Result<Exit> {
    match args.first().map(String::as_str) {
        None => {
            writeln!(err, "error: missing subcommand")?;
            writeln!(err, "{USAGE}")?;
            Ok(Exit::Usage)
        }
        Some("help" | "--help" | "-h") => {
            writeln!(out, "{USAGE}")?;
            Ok(Exit::Ok)
        }
        Some("validate") => validate::run(&args[1..], out, err),
        Some("export") => export::run(&args[1..], out, err),
        Some("intake") => intake::run(&args[1..], out, err),
        Some("ingest") => stub_ingest(err),
        Some("snapshot") => stub_snapshot(err),
        Some(other) => {
            writeln!(err, "error: unknown subcommand '{other}'")?;
            writeln!(err, "{USAGE}")?;
            Ok(Exit::Usage)
        }
    }
}

fn stub_ingest(err: &mut dyn Write) -> std::io::Result<Exit> {
    writeln!(
        err,
        "cn ingest is not available: ingest routing semantics and the importer \
         contract are parked behind the G-RAT human gate (DECISIONS.md D-044.2). \
         A human must answer G-RAT before any ingest behavior ships."
    )?;
    Ok(Exit::Usage)
}

fn stub_snapshot(err: &mut dyn Write) -> std::io::Result<Exit> {
    writeln!(
        err,
        "cn snapshot is not available: it depends on the Phase 2 snapshot data \
         envelope (PLAN_1.0.md P2.3). The subcommand lands once that envelope exists."
    )?;
    Ok(Exit::Usage)
}
