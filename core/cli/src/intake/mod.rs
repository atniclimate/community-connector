//! `cn intake`: the native durable owner of the intake queue (ADR-005 D4;
//! docs/blueprints/intake-pipeline.md sections 1-2 and 9 step 5).
//!
//! The CLI executes I/O for the pure cn-ingest verdicts; every trust
//! decision - admission, planning, authorization, fold acceptance - lives
//! in the core crates (I2). This module is argument marshalling and
//! dispatch only.

mod apply;
mod queue;

use std::io::Write;

use crate::Exit;

const USAGE: &str = "usage: cn intake apply --queue <queue-root> --ops <ops.jsonl> \
--group <group-uuid> --facilitator <person-uuid> [--kind <kind-id>]

Runs the intake apply transaction as the queue's single native mutator
(ADR-005 D4): startup recovery over the crash-state table, decision-inbox
admission (dedup, generation+state CAS, legal transitions), approval
planning, the idempotent durable append, completion transaction events,
and the machine-readable run report (JSON on stdout, I12).

The queue root must lie OUTSIDE any git worktree or cloud-synced
directory; apply refuses to run otherwise. A single-instance lock at the
queue root serializes native mutators; a second concurrent apply refuses
to run. --kind is the default entity kind for approvals whose payload
carries no `kind` field (pilot form: person).";

pub(crate) fn run(
    args: &[String],
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> std::io::Result<Exit> {
    match args.first().map(String::as_str) {
        Some("apply") => apply::run(&args[1..], out, err),
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
