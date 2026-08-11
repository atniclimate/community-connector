//! `emit-approve-decision`: an OPT-IN test harness that writes a valid approve
//! `DecisionMessage` into a queue's `decisions/` inbox (blueprint
//! docs/blueprints/intake-relay.md section 8 step 9; ADR-005 D4).
//!
//! The facilitator wizard that normally authors an approve decision is
//! TypeScript/FSA and browser-driven (blueprint step 5). The automated
//! form-to-graph rehearsal (`scripts/e2e-remote-intake.ps1`) has no browser, so
//! it needs another way to produce the exact decision file `cn intake apply`
//! consumes. This example is that way: it reads the single staged record a pull
//! left in the queue, binds an approve decision to that record's checksum (the
//! same digest binding `admit` verifies), and drops it in `decisions/` under the
//! `<record_id>.<decision_id>.json` name apply scans for.
//!
//! It is an EXAMPLE on purpose (ADR-005 D4): the shipped `cn` binary stays
//! create-only for decisions via the wizard; no decision-emit subcommand is
//! added to it. This harness is built and run only by the e2e script, never
//! wired into `check-all`. It emits the SAME `DecisionMessage` shape the
//! `intake_apply.rs` integration battery constructs by hand, so the digest
//! binding, `expected_review_state`, and `expected_decision_generation` match a
//! freshly staged, never-decided record (Pending, generation 0).

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use cn_ingest::{
    DecisionMessage, DecisionType, QueueRecord, ReviewState, decision_message_version,
};
use cn_model::Timestamp;
use uuid::Uuid;

fn main() {
    if let Err(message) = run() {
        eprintln!("emit-approve-decision: error: {message}");
        std::process::exit(1);
    }
}

struct Args {
    queue: PathBuf,
    reviewer: String,
    decision_id: String,
    decided_at_ms: i64,
}

fn run() -> Result<(), String> {
    let args = parse_args()?;

    // Exactly one staged record: a pull run stages one QueueRecord (+ its
    // pending sidecar) per receipt, and the e2e submits exactly one envelope.
    // More than one here means the queue was reused or the pull staged extras -
    // either way, "which record?" must never be guessed (I3).
    let record_path = sole_record_file(&args.queue)?;
    let record: QueueRecord = read_json(&record_path)?;

    let decision = DecisionMessage {
        queue_record_version: decision_message_version(),
        decision_id: args.decision_id.clone(),
        record_id: record.record_id.clone(),
        // The digest binding `admit` re-checks: the decision approves THIS
        // record's exact bytes, never a substituted payload (round-1 F8, I6).
        payload_digest: record.record_checksum.clone(),
        // A freshly staged record has never been decided: pending, generation 0
        // (mirrors the intake_apply.rs happy-path decision).
        expected_review_state: ReviewState::Pending,
        expected_decision_generation: 0,
        decision: DecisionType::Approve,
        reviewer: args.reviewer.clone(),
        decided_at: Timestamp(args.decided_at_ms),
    };

    // Write into the create-only decision inbox exactly as the FSA adapter and
    // the intake_apply battery do: `decisions/<record_id>.<decision_id>.json`.
    let dir = args.queue.join("decisions");
    fs::create_dir_all(&dir)
        .map_err(|io_err| format!("cannot create decisions dir '{}': {io_err}", dir.display()))?;
    let file = dir.join(format!("{}.{}.json", record.record_id, args.decision_id));
    let bytes = serde_json::to_vec_pretty(&decision)
        .map_err(|render_err| format!("cannot serialize decision: {render_err}"))?;
    fs::write(&file, &bytes)
        .map_err(|io_err| format!("cannot write '{}': {io_err}", file.display()))?;

    // Machine-readable summary for the orchestrator (stdout), so the script
    // asserts on the exact record/decision it just approved.
    let summary = serde_json::json!({
        "record_id": record.record_id,
        "decision_id": args.decision_id,
        "reviewer": args.reviewer,
        "payload_digest": record.record_checksum,
        "decision_file": file.display().to_string(),
    });
    println!(
        "{}",
        serde_json::to_string(&summary).map_err(|err| err.to_string())?
    );
    Ok(())
}

fn parse_args() -> Result<Args, String> {
    let mut queue = None;
    let mut reviewer = None;
    let mut decision_id = None;
    let mut decided_at_ms = None;
    let mut iter = std::env::args().skip(1);
    while let Some(flag) = iter.next() {
        match flag.as_str() {
            "--queue" => queue = Some(PathBuf::from(next_value(&mut iter, &flag)?)),
            "--reviewer" => reviewer = Some(next_value(&mut iter, &flag)?),
            "--decision-id" => decision_id = Some(next_value(&mut iter, &flag)?),
            "--decided-at" => {
                let raw = next_value(&mut iter, &flag)?;
                decided_at_ms = Some(
                    raw.parse::<i64>()
                        .map_err(|_| format!("--decided-at '{raw}' is not an integer (ms)"))?,
                );
            }
            other => return Err(format!("unknown argument '{other}'")),
        }
    }
    Ok(Args {
        queue: queue.ok_or_else(|| "--queue is required".to_string())?,
        reviewer: reviewer.ok_or_else(|| "--reviewer is required".to_string())?,
        decision_id: decision_id.unwrap_or_else(|| Uuid::now_v7().to_string()),
        decided_at_ms: match decided_at_ms {
            Some(ms) => ms,
            None => unix_now_ms()?,
        },
    })
}

fn next_value(iter: &mut impl Iterator<Item = String>, flag: &str) -> Result<String, String> {
    iter.next().ok_or_else(|| format!("{flag} needs a value"))
}

/// Finds the single `*.record.json` in the queue root, erroring loudly on zero
/// or more than one (the e2e stages exactly one).
fn sole_record_file(queue: &Path) -> Result<PathBuf, String> {
    let mut found: Vec<PathBuf> = Vec::new();
    let entries = fs::read_dir(queue)
        .map_err(|io_err| format!("cannot read queue '{}': {io_err}", queue.display()))?;
    for entry in entries {
        let path = entry
            .map_err(|io_err| format!("cannot read a queue entry: {io_err}"))?
            .path();
        if path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.ends_with(".record.json"))
        {
            found.push(path);
        }
    }
    match found.len() {
        1 => Ok(found.pop().expect("len checked")),
        0 => Err(format!(
            "no *.record.json in '{}': did the pull stage a record?",
            queue.display()
        )),
        n => Err(format!(
            "{n} *.record.json files in '{}': expected exactly one staged record",
            queue.display()
        )),
    }
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, String> {
    let bytes =
        fs::read(path).map_err(|io_err| format!("cannot read '{}': {io_err}", path.display()))?;
    serde_json::from_slice(&bytes)
        .map_err(|parse_err| format!("cannot parse '{}': {parse_err}", path.display()))
}

fn unix_now_ms() -> Result<i64, String> {
    let elapsed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|clock_err| format!("system clock is before the unix epoch: {clock_err}"))?;
    i64::try_from(elapsed.as_millis())
        .map_err(|range_err| format!("system time overflows the op clock: {range_err}"))
}
