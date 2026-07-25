//! The `cn intake apply` run (ADR-005 D4; blueprint section 2's "the
//! durable owner" and section 9 step 5).
//!
//! Order inside one locked run: open the durable store -> scan the queue ->
//! approval recovery FIRST (before any other queue work) -> the rest of the
//! crash-state table -> tombstone reconciliation -> decision admission in
//! deterministic order -> the I12 run report.
//!
//! Halting rule: the lost-decision-state and binding-mismatch rows, a
//! failed sidecar rewrite, and a corrupt persisted plan HALT the run before
//! any decision is admitted (stop-the-line; nothing is guessed, ADR-005
//! D4). Quarantines, stale/illegal decisions, and id-reuse conflicts are
//! recorded loudly and the run continues; any of them still yields a
//! failure exit code so attention is unmissable.

use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::Serialize;
use serde_json::json;

use cn_ingest::{
    AdmissionVerdict, ApprovalPlan, DecisionMessage, DecisionOutcome, DecisionType, FoundRecord,
    FoundSidecar, HistoryEntry, PlanContext, QueueRecord, RecoveryAction, ReviewSidecar,
    ReviewState, TransactionKind, admit, apply_verdict, classify, plan_approval,
    record_transaction,
};
use cn_model::{GroupId, KindId, PersonId, Timestamp};
use cn_perm::PermAuthorizer;
use cn_store::{
    BatchEntry, BatchFailure, BatchMode, DurableOpIndex, GroupState, OpDisposition, OpLog,
    Operation, StoreReport, append_batch_idempotent, fold,
};

use super::queue::{self, QueuePaths};
use crate::Exit;

struct ApplyArgs {
    queue: String,
    ops: String,
    group: String,
    facilitator: String,
    kind: String,
}

/// Live queue pair the admission loop mutates.
struct Live {
    record: QueueRecord,
    sidecar: ReviewSidecar,
}

#[derive(Serialize)]
struct RunReport {
    queue_root: String,
    group_id: String,
    scanned: ScanCounts,
    recovery: Vec<RecoveryEntry>,
    tombstone_anomalies: Vec<String>,
    decisions: Vec<DecisionEntry>,
    halts: Vec<String>,
    warnings: Vec<String>,
    review_states: BTreeMap<String, usize>,
    ops_appended: usize,
}

#[derive(Serialize)]
struct ScanCounts {
    payload_records: usize,
    decision_files: usize,
    tombstones: usize,
    temp_discarded: usize,
}

#[derive(Serialize)]
struct RecoveryEntry {
    record_id: String,
    action: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    detail: Option<String>,
}

#[derive(Serialize)]
struct DecisionEntry {
    decision_id: String,
    record_id: String,
    outcome: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    transaction: Option<TransactionEntry>,
}

#[derive(Serialize, Clone)]
struct TransactionEntry {
    kind: String,
    resulting_state: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    detail: Option<String>,
}

pub(crate) fn run(
    args: &[String],
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> std::io::Result<Exit> {
    let parsed = match parse_args(args) {
        Ok(parsed) => parsed,
        Err(message) => {
            writeln!(err, "error: {message}")?;
            super::usage_to(err)?;
            return Ok(Exit::Usage);
        }
    };
    let root = match queue::refuse_unsafe_root(Path::new(&parsed.queue)) {
        Ok(root) => root,
        Err(message) => {
            writeln!(err, "error: {message}")?;
            return Ok(Exit::Failure);
        }
    };
    let paths = QueuePaths::new(&root);
    let _lock = match queue::acquire_lock(&paths) {
        Ok(lock) => lock,
        Err(message) => {
            writeln!(err, "error: {message}")?;
            return Ok(Exit::Failure);
        }
    };
    match run_apply(&parsed, &paths) {
        Ok((report, exit)) => {
            let rendered = serde_json::to_string_pretty(&report)
                .unwrap_or_else(|render_err| format!("{{\"render_error\":\"{render_err}\"}}"));
            writeln!(out, "{rendered}")?;
            if exit != Exit::Ok {
                writeln!(
                    err,
                    "error: the run report above records halts, conflicts, or anomalies \
                     needing facilitator attention"
                )?;
            }
            Ok(exit)
        }
        Err(message) => {
            writeln!(err, "error: {message}")?;
            Ok(Exit::Failure)
        }
    }
}

fn parse_args(args: &[String]) -> Result<ApplyArgs, String> {
    let mut queue = None;
    let mut ops = None;
    let mut group = None;
    let mut facilitator = None;
    let mut kind = None;
    let mut iter = args.iter();
    while let Some(flag) = iter.next() {
        let slot = match flag.as_str() {
            "--queue" => &mut queue,
            "--ops" => &mut ops,
            "--group" => &mut group,
            "--facilitator" => &mut facilitator,
            "--kind" => &mut kind,
            other => return Err(format!("unknown intake apply argument '{other}'")),
        };
        let value = iter.next().ok_or_else(|| format!("{flag} needs a value"))?;
        if slot.replace(value.clone()).is_some() {
            return Err(format!("{flag} given more than once"));
        }
    }
    Ok(ApplyArgs {
        queue: queue.ok_or_else(|| "--queue is required".to_string())?,
        ops: ops.ok_or_else(|| "--ops is required".to_string())?,
        group: group.ok_or_else(|| "--group is required".to_string())?,
        facilitator: facilitator.ok_or_else(|| "--facilitator is required".to_string())?,
        kind: kind.unwrap_or_else(|| cn_ingest::DEFAULT_PILOT_KIND.to_string()),
    })
}

fn run_apply(args: &ApplyArgs, paths: &QueuePaths) -> Result<(RunReport, Exit), String> {
    let group_id: GroupId = args
        .group
        .parse()
        .map_err(|_| format!("--group '{}' is not a valid group uuid", args.group))?;
    let facilitator: PersonId = args.facilitator.parse().map_err(|_| {
        format!(
            "--facilitator '{}' is not a valid person uuid",
            args.facilitator
        )
    })?;
    let default_kind = KindId::new(args.kind.clone())
        .map_err(|_| format!("--kind '{}' is not a valid kind id", args.kind))?;
    let now_ms = crate::export::unix_now_ms()?;

    // The durable store: replay, index, fold. The folded state's group and
    // template are the source of truth (I2) - apply takes no template file.
    let ops_path = PathBuf::from(&args.ops);
    let (mut log, replayed, open_report) = OpLog::open(&ops_path)
        .map_err(|store_err| format!("cannot open ops log '{}': {store_err}", args.ops))?;
    let mut index = DurableOpIndex::from_ops(&replayed)
        .map_err(|store_err| format!("cannot index ops log: {store_err}"))?;
    let (mut state, fold_report) = fold(replayed);
    let mut warnings = store_findings(&open_report, "log-open");
    warnings.extend(store_findings(&fold_report, "replay-fold"));

    let (template, template_version) = {
        let group = state
            .group
            .as_ref()
            .ok_or("ops log holds no group; run against an initialized group log")?;
        if group.id != group_id {
            return Err(format!(
                "--group {group_id} does not match the log's group {}",
                group.id
            ));
        }
        (
            state
                .template
                .clone()
                .ok_or("ops log holds no group template; cannot plan approvals")?,
            group.template_version.clone(),
        )
    };

    if !queue::dir_sync_supported() {
        warnings.push(
            "directory-handle flush is unavailable on this platform; staging proceeds \
             with this recorded WARN (ADR-005 D4 degraded-platform row, I12)"
                .to_string(),
        );
    }

    let scan = queue::scan(paths)?;
    for name in &scan.unrecognized {
        warnings.push(format!("unrecognized file '{name}' at the queue root"));
    }
    let mut attention = false;
    for (name, reason) in &scan.unreadable_decisions {
        warnings.push(format!(
            "decision file '{name}' is unreadable ({reason}); left in place for \
             facilitator disposition (I3)"
        ));
        attention = true;
    }
    let scanned = ScanCounts {
        payload_records: scan.records.len(),
        decision_files: scan.decisions.len() + scan.unreadable_decisions.len(),
        tombstones: scan.tombstones.len(),
        temp_discarded: scan.temp_discarded,
    };

    // Crash-state classification (cn-ingest owns the table); approval
    // recovery runs before any other queue work (ADR-005 D4).
    let mut pending_by_record: BTreeMap<String, usize> = BTreeMap::new();
    for (_, message) in &scan.decisions {
        *pending_by_record
            .entry(message.record_id.clone())
            .or_insert(0) += 1;
    }
    let mut classified: Vec<(RecoveryAction, FoundRecord)> = Vec::new();
    for (record_id, scanned_record) in scan.records {
        let found = FoundRecord {
            record_id: record_id.clone(),
            payload: scanned_record.payload,
            payload_corrupt: scanned_record.payload_corrupt,
            sidecar: scanned_record.sidecar,
            marker_present: scanned_record.marker_present,
            pending_decisions: pending_by_record.get(&record_id).copied().unwrap_or(0),
        };
        let action = classify(&found).map_err(|ingest_err| ingest_err.to_string())?;
        classified.push((action, found));
    }
    classified
        .sort_by_key(|(action, _)| !matches!(action, RecoveryAction::RunApprovalRecovery { .. }));

    let mut live: BTreeMap<String, Live> = BTreeMap::new();
    let mut recovery = Vec::new();
    let mut halts = Vec::new();
    let mut ops_appended = 0usize;
    for (action, found) in classified {
        match action {
            RecoveryAction::None { record_id } => {
                insert_live(&mut live, record_id, found);
            }
            RecoveryAction::ProceedWithAdmission { record_id } => {
                recovery.push(RecoveryEntry {
                    record_id: record_id.clone(),
                    action: "proceed_with_admission".to_string(),
                    detail: Some(
                        "marker present with unconsumed decisions; benign transient".to_string(),
                    ),
                });
                insert_live(&mut live, record_id, found);
            }
            RecoveryAction::ReportMarkerAnomaly { record_id } => {
                warnings.push(format!(
                    "record {record_id}: review-begun marker present with no decision \
                     history and no decision file; stays pending - re-decide in the \
                     wizard if a decision was intended (I12)"
                ));
                recovery.push(RecoveryEntry {
                    record_id: record_id.clone(),
                    action: "report_marker_anomaly".to_string(),
                    detail: None,
                });
                insert_live(&mut live, record_id, found);
            }
            RecoveryAction::ReconstructPendingSidecar { record_id } => {
                let record = found
                    .payload
                    .ok_or_else(|| format!("record {record_id}: classify invariant broken"))?;
                let sidecar =
                    ReviewSidecar::initial(&record).map_err(|ingest_err| ingest_err.to_string())?;
                write_sidecar(paths, &sidecar)?;
                recovery.push(RecoveryEntry {
                    record_id: record_id.clone(),
                    action: "reconstruct_pending_sidecar".to_string(),
                    detail: Some("initial pending state only; no history recreated".to_string()),
                });
                live.insert(record_id, Live { record, sidecar });
            }
            RecoveryAction::QuarantineCorrupt { record_id } => {
                queue::quarantine_pair(paths, &record_id)?;
                warnings.push(format!(
                    "record {record_id}: checksum-failing pair moved to corrupt/ \
                     (retained, never trusted; loud typed error, I3)"
                ));
                recovery.push(RecoveryEntry {
                    record_id,
                    action: "quarantine_corrupt".to_string(),
                    detail: None,
                });
                attention = true;
            }
            RecoveryAction::HaltLostDecisionState { record_id } => {
                halts.push(format!(
                    "record {record_id}: review-begun marker present without a readable \
                     sidecar - lost decision state; a decision history is never silently \
                     recreated (ADR-005 D4)"
                ));
            }
            RecoveryAction::HaltBindingMismatch { record_id } => {
                halts.push(format!(
                    "record {record_id}: sidecar binding does not match its payload; both \
                     files retained for facilitator investigation, no automatic repair \
                     (ADR-005 D4)"
                ));
            }
            RecoveryAction::RunApprovalRecovery { record_id } => {
                let record = found
                    .payload
                    .ok_or_else(|| format!("record {record_id}: classify invariant broken"))?;
                let FoundSidecar::Valid(sidecar) = found.sidecar else {
                    return Err(format!("record {record_id}: classify invariant broken"));
                };
                let mut sidecar = *sidecar;
                match recover_approval(
                    paths,
                    &record_id,
                    &mut sidecar,
                    &mut log,
                    &mut index,
                    &mut state,
                    now_ms,
                ) {
                    Ok((transaction, appended)) => {
                        ops_appended += appended;
                        recovery.push(RecoveryEntry {
                            record_id: record_id.clone(),
                            action: "approval_recovery".to_string(),
                            detail: Some(format!(
                                "{} -> {}",
                                transaction.kind, transaction.resulting_state
                            )),
                        });
                        live.insert(record_id, Live { record, sidecar });
                    }
                    Err(message) => halts.push(message),
                }
            }
        }
    }

    // Startup tombstone reconciliation: a consumed tombstone must appear in
    // a decision event of its record (I12; never silently deleted).
    let mut tombstone_anomalies = Vec::new();
    for (name, parsed) in &scan.tombstones {
        match parsed {
            Err(reason) => {
                tombstone_anomalies.push(format!("unreadable tombstone '{name}': {reason}"));
            }
            Ok(message) => {
                let recorded = live
                    .get(&message.record_id)
                    .is_some_and(|entry| has_decision_event(&entry.sidecar, &message.decision_id));
                if !recorded {
                    tombstone_anomalies.push(format!(
                        "tombstone '{name}': decision {} appears in no decision event of \
                         record {} - impossible under the ordering rule; investigate (I12)",
                        message.decision_id, message.record_id
                    ));
                }
            }
        }
    }

    let mut decisions = Vec::new();
    if halts.is_empty() {
        // Deterministic inbox order: (decided_at, decision_id).
        let mut messages = scan.decisions;
        messages.sort_by(|a, b| {
            (a.1.decided_at, a.1.decision_id.as_str())
                .cmp(&(b.1.decided_at, b.1.decision_id.as_str()))
        });
        for (file, message) in messages {
            match consume_decision(
                paths,
                &file,
                &message,
                &mut live,
                &template,
                &template_version,
                group_id,
                facilitator,
                &default_kind,
                now_ms,
                &mut log,
                &mut index,
                &mut state,
                &mut warnings,
            ) {
                Ok(Consumed::Entry(entry, appended)) => {
                    if matches!(
                        entry.outcome.as_str(),
                        "id_reuse_conflict" | "binding_mismatch" | "unknown_record"
                    ) {
                        attention = true;
                    }
                    ops_appended += appended;
                    decisions.push(entry);
                }
                Err(message) => {
                    halts.push(message);
                    break;
                }
            }
        }
    }

    let mut review_states: BTreeMap<String, usize> = BTreeMap::new();
    for entry in live.values() {
        *review_states
            .entry(state_name(entry.sidecar.review_state).to_string())
            .or_insert(0) += 1;
    }

    let failure = !halts.is_empty() || attention || !tombstone_anomalies.is_empty();
    let report = RunReport {
        queue_root: paths.root().display().to_string(),
        group_id: group_id.to_string(),
        scanned,
        recovery,
        tombstone_anomalies,
        decisions,
        halts,
        warnings,
        review_states,
        ops_appended,
    };
    Ok((report, if failure { Exit::Failure } else { Exit::Ok }))
}

enum Consumed {
    Entry(DecisionEntry, usize),
}

#[allow(clippy::too_many_arguments)]
fn consume_decision(
    paths: &QueuePaths,
    file: &Path,
    message: &DecisionMessage,
    live: &mut BTreeMap<String, Live>,
    template: &cn_schema::GroupTemplate,
    template_version: &semver::Version,
    group_id: GroupId,
    facilitator: PersonId,
    default_kind: &KindId,
    now_ms: i64,
    log: &mut OpLog,
    index: &mut DurableOpIndex,
    state: &mut GroupState,
    warnings: &mut Vec<String>,
) -> Result<Consumed, String> {
    let entry = |outcome: &str, transaction: Option<TransactionEntry>| DecisionEntry {
        decision_id: message.decision_id.clone(),
        record_id: message.record_id.clone(),
        outcome: outcome.to_string(),
        transaction,
    };

    let Some(target) = live.get_mut(&message.record_id) else {
        warnings.push(format!(
            "decision {} targets unknown, quarantined, or halted record {}; file left \
             in place (I3)",
            message.decision_id, message.record_id
        ));
        return Ok(Consumed::Entry(entry("unknown_record", None), 0));
    };

    let verdict = admit(&target.sidecar, message).map_err(|ingest_err| ingest_err.to_string())?;
    match &verdict {
        AdmissionVerdict::WritelessReplay => {
            // The original durable entry is the retirement proof; no write.
            queue::retire_decision(paths, file)?;
            Ok(Consumed::Entry(entry("writeless_replay", None), 0))
        }
        AdmissionVerdict::IdReuseConflict => {
            warnings.push(format!(
                "decision id {} reused with DIFFERENT bytes for record {} - loud typed \
                 conflict (I3), never a replay; file left for facilitator disposition",
                message.decision_id, message.record_id
            ));
            Ok(Consumed::Entry(entry("id_reuse_conflict", None), 0))
        }
        AdmissionVerdict::BindingMismatch => {
            warnings.push(format!(
                "decision {} fails its payload-digest binding against record {}; file \
                 left for facilitator disposition (I3)",
                message.decision_id, message.record_id
            ));
            Ok(Consumed::Entry(entry("binding_mismatch", None), 0))
        }
        AdmissionVerdict::Stale { .. } | AdmissionVerdict::Illegal { .. } => {
            let outcome = if matches!(verdict, AdmissionVerdict::Stale { .. }) {
                "stale"
            } else {
                "illegal"
            };
            queue::ensure_marker(paths, &message.record_id)?;
            apply_verdict(&mut target.sidecar, &verdict)
                .map_err(|ingest_err| ingest_err.to_string())?;
            write_sidecar(paths, &target.sidecar)?;
            queue::retire_decision(paths, file)?;
            Ok(Consumed::Entry(entry(outcome, None), 0))
        }
        AdmissionVerdict::Admit { .. } => {
            queue::ensure_marker(paths, &message.record_id)?;
            let mut planned: Option<ApprovalPlan> = None;
            if matches!(message.decision, DecisionType::Approve) {
                let context = PlanContext {
                    group_id,
                    facilitator,
                    kind: kind_for(&target.record, default_kind, warnings),
                    now_ms,
                    template_version: template_version.clone(),
                };
                let plan = plan_approval(&target.record, template, &context, &mut || {
                    uuid::Uuid::now_v7()
                })
                .map_err(|ingest_err| ingest_err.to_string())?;
                target.sidecar.plan = Some(plan.plan_ref.clone());
                target.sidecar.validation_report = Some(
                    serde_json::to_string(&plan.validation)
                        .map_err(|render_err| render_err.to_string())?,
                );
                planned = Some(plan);
            }
            // Admission + history + plan + intent: ONE atomic sidecar
            // replace (apply_verdict seals over the plan set above).
            apply_verdict(&mut target.sidecar, &verdict)
                .map_err(|ingest_err| ingest_err.to_string())?;
            write_sidecar(paths, &target.sidecar)?;
            queue::retire_decision(paths, file)?;

            let mut transaction = None;
            let mut appended_count = 0;
            if let Some(plan) = planned {
                let digests = plan.plan_ref.per_op_digests.clone();
                let (summary, appended) = execute_transaction(
                    paths,
                    &mut target.sidecar,
                    &message.decision_id,
                    plan.ops,
                    &digests,
                    BatchMode::FirstAttempt,
                    log,
                    index,
                    state,
                    now_ms,
                )?;
                appended_count = appended;
                transaction = Some(summary);
            }
            Ok(Consumed::Entry(
                entry("admitted", transaction),
                appended_count,
            ))
        }
    }
}

/// Runs the approval-recovery rule for a sidecar found in `approved_intent`
/// (ADR-005 D4): the persisted plan's ops go back through the seam in
/// recovery mode - no re-authorization; the durable intent marker governs.
fn recover_approval(
    paths: &QueuePaths,
    record_id: &str,
    sidecar: &mut ReviewSidecar,
    log: &mut OpLog,
    index: &mut DurableOpIndex,
    state: &mut GroupState,
    now_ms: i64,
) -> Result<(TransactionEntry, usize), String> {
    let plan = sidecar.plan.clone().ok_or_else(|| {
        format!("record {record_id}: approved_intent with no persisted plan; halting (I3)")
    })?;
    if plan.ops.len() != plan.per_op_digests.len() || plan.ops.is_empty() {
        return Err(format!(
            "record {record_id}: persisted plan is malformed ({} ops, {} digests); halting",
            plan.ops.len(),
            plan.per_op_digests.len()
        ));
    }
    let ops: Vec<Operation> = plan
        .ops
        .iter()
        .map(|value| serde_json::from_value(value.clone()))
        .collect::<Result<_, _>>()
        .map_err(|parse_err| {
            format!("record {record_id}: persisted plan ops do not parse ({parse_err}); halting")
        })?;
    let decision_id = admitting_decision_id(sidecar).ok_or_else(|| {
        format!(
            "record {record_id}: approved_intent with no admitting approve decision \
             event; halting"
        )
    })?;
    execute_transaction(
        paths,
        sidecar,
        &decision_id,
        ops,
        &plan.per_op_digests,
        BatchMode::RecoveryUnderIntent,
        log,
        index,
        state,
        now_ms,
    )
}

/// Pushes the plan through the durable seam and records the outcome as the
/// linked transaction event (ADR-005 D4 step 3). A failed sidecar rewrite
/// is a halt, not a warning.
#[allow(clippy::too_many_arguments)]
fn execute_transaction(
    paths: &QueuePaths,
    sidecar: &mut ReviewSidecar,
    decision_id: &str,
    ops: Vec<Operation>,
    digests: &[String],
    mode: BatchMode,
    log: &mut OpLog,
    index: &mut DurableOpIndex,
    state: &mut GroupState,
    now_ms: i64,
) -> Result<(TransactionEntry, usize), String> {
    let batch: Vec<BatchEntry> = ops
        .into_iter()
        .zip(digests.iter())
        .map(|(op, digest)| BatchEntry {
            op,
            digest: digest.clone(),
        })
        .collect();
    let mut scratch = StoreReport::default();
    let outcome = append_batch_idempotent(
        log,
        index,
        state,
        &PermAuthorizer,
        &batch,
        mode,
        &mut scratch,
    );

    let (kind, resulting_state, detail, appended) = match outcome {
        Ok(dispositions) => {
            let appended = dispositions
                .iter()
                .filter(|(_, disposition)| *disposition == OpDisposition::AbsentAppended)
                .count();
            let detail = json!({
                "appended": appended,
                "already_durable": dispositions.len() - appended,
            })
            .to_string();
            (
                TransactionKind::IntentCompleted,
                ReviewState::Approved,
                detail,
                appended,
            )
        }
        Err(BatchFailure::Denied { op_id, denial }) => {
            require_first_attempt(mode, "authorization denial")?;
            (
                TransactionKind::PreflightFailed,
                ReviewState::Pending,
                json!({ "op_id": op_id.to_string(), "code": denial.code, "message": denial.message })
                    .to_string(),
                0,
            )
        }
        Err(BatchFailure::WouldQuarantine { op_id }) => {
            require_first_attempt(mode, "would-quarantine")?;
            (
                TransactionKind::PreflightFailed,
                ReviewState::Pending,
                json!({ "op_id": op_id.to_string(), "code": "would_quarantine" }).to_string(),
                0,
            )
        }
        Err(BatchFailure::DigestConflict { op_id }) => (
            TransactionKind::DurableConflict,
            ReviewState::Failed,
            json!({ "op_id": op_id.to_string(), "code": "durable_conflict" }).to_string(),
            0,
        ),
        Err(BatchFailure::DurableInconsistency { op_id }) => (
            TransactionKind::DurableInconsistency,
            ReviewState::Failed,
            json!({ "op_id": op_id.to_string(), "code": "durable_inconsistency" }).to_string(),
            0,
        ),
        Err(BatchFailure::PlanDigestMismatch { op_id }) => {
            return Err(format!(
                "plan digest mismatch for op {op_id}: the persisted plan is corrupt; \
                 halting (I3)"
            ));
        }
        Err(BatchFailure::Store(store_err)) => {
            return Err(format!(
                "durable store failure during the approval transaction: {store_err}; halting"
            ));
        }
    };

    record_transaction(
        sidecar,
        decision_id,
        kind,
        resulting_state,
        Timestamp(now_ms),
        Some(detail.clone()),
    )
    .map_err(|ingest_err| ingest_err.to_string())?;
    write_sidecar(paths, sidecar).map_err(|write_err| {
        format!(
            "sidecar rewrite failed after the {} transaction event: {write_err}; a \
             repeatedly encountered approved_intent is a stop condition (ADR-005 D4)",
            kind_name(kind)
        )
    })?;
    Ok((
        TransactionEntry {
            kind: kind_name(kind).to_string(),
            resulting_state: state_name(resulting_state).to_string(),
            detail: Some(detail),
        },
        appended,
    ))
}

fn require_first_attempt(mode: BatchMode, what: &str) -> Result<(), String> {
    if mode == BatchMode::RecoveryUnderIntent {
        return Err(format!(
            "impossible {what} during recovery under intent (the seam skips preflight); \
             halting"
        ));
    }
    Ok(())
}

fn insert_live(live: &mut BTreeMap<String, Live>, record_id: String, found: FoundRecord) {
    if let (Some(record), FoundSidecar::Valid(sidecar)) = (found.payload, found.sidecar) {
        live.insert(
            record_id,
            Live {
                record,
                sidecar: *sidecar,
            },
        );
    }
}

fn has_decision_event(sidecar: &ReviewSidecar, decision_id: &str) -> bool {
    sidecar.history.iter().any(|entry| {
        matches!(entry, HistoryEntry::Decision { decision_id: recorded, .. } if recorded == decision_id)
    })
}

fn admitting_decision_id(sidecar: &ReviewSidecar) -> Option<String> {
    sidecar.history.iter().rev().find_map(|entry| match entry {
        HistoryEntry::Decision {
            decision_id,
            resulting_state,
            outcome,
            ..
        } if *resulting_state == ReviewState::ApprovedIntent
            && *outcome == DecisionOutcome::Admitted =>
        {
            Some(decision_id.clone())
        }
        _ => None,
    })
}

/// Resolves the entity kind via the shared cn-ingest rule (payload `kind`
/// wins, else the CLI default), surfacing any fallback warning (I12).
fn kind_for(record: &QueueRecord, default_kind: &KindId, warnings: &mut Vec<String>) -> KindId {
    let (kind, warning) = cn_ingest::resolve_kind(record, default_kind);
    warnings.extend(warning);
    kind
}

fn write_sidecar(paths: &QueuePaths, sidecar: &ReviewSidecar) -> Result<(), String> {
    let bytes = serde_json::to_vec_pretty(sidecar)
        .map_err(|render_err| format!("cannot serialize sidecar: {render_err}"))?;
    queue::write_atomic(&paths.sidecar(&sidecar.record_id), &bytes)
}

fn store_findings(report: &StoreReport, stage: &str) -> Vec<String> {
    let mut findings: Vec<String> = report
        .warnings
        .iter()
        .map(|finding| format!("{stage}: {}: {}", finding.code, finding.message))
        .collect();
    if !report.quarantined.is_empty() {
        findings.push(format!(
            "{stage}: {} op(s) quarantined during replay",
            report.quarantined.len()
        ));
    }
    findings
}

fn state_name(state: ReviewState) -> &'static str {
    match state {
        ReviewState::Pending => "pending",
        ReviewState::ApprovedIntent => "approved_intent",
        ReviewState::Approved => "approved",
        ReviewState::Rejected => "rejected",
        ReviewState::Failed => "failed",
    }
}

fn kind_name(kind: TransactionKind) -> &'static str {
    match kind {
        TransactionKind::IntentCompleted => "intent_completed",
        TransactionKind::PreflightFailed => "preflight_failed",
        TransactionKind::DurableConflict => "durable_conflict",
        TransactionKind::DurableInconsistency => "durable_inconsistency",
    }
}
