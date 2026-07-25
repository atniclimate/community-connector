//! The `cn intake apply` run (ADR-005 D4; blueprint section 2's "the
//! durable owner" and section 9 step 5; amended per the 2026-07-25
//! adversarial round 1).
//!
//! Order inside one locked run: open the durable store -> scan the queue ->
//! classify the whole crash-state table -> STOP-THE-LINE if any halt row
//! classified (nothing executes) -> approval recovery first -> the rest of
//! the recovery actions -> tombstone reconciliation -> decision admission
//! in deterministic order -> the I12 run report.
//!
//! Halting rule (D-070.2 as amended): halt-class classifications
//! (lost-decision-state, binding mismatch, marker-with-nothing) stop the
//! run BEFORE any mutation; a failed approval recovery, failed sidecar
//! rewrite, or corrupt persisted plan stops the run at that point with
//! nothing further executed. Quarantines, stale/illegal decisions, and
//! id-reuse conflicts are recorded loudly and the run continues; any of
//! them still yields a failure exit code so attention is unmissable.
//!
//! Recovery mode selection (round-1 F1): an ALL-ABSENT plan re-runs the
//! FULL first-attempt step - preflight authorization, fold acceptance,
//! and the validation gate included (ADR-005 D4 "re-run step 2"); only a
//! plan with durable presence completes under the intent marker without
//! re-authorization.

use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::Serialize;
use serde_json::json;

use cn_ingest::{
    AdmissionVerdict, ApprovalPlan, DecisionMessage, DecisionOutcome, DecisionType, FoundRecord,
    FoundSidecar, HistoryEntry, PlanContext, QueueRecord, RecoveryAction, ReviewSidecar,
    ReviewState, TransactionKind, admit, apply_verdict, classify, plan_approval,
    record_transaction, validate_record, validate_submission, verify_persisted_plan,
};
use cn_model::{GroupId, KindId, OpId, PersonId, Timestamp};
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

/// Everything the transaction executor needs from the run context.
struct TxContext<'a> {
    paths: &'a QueuePaths,
    log: &'a mut OpLog,
    index: &'a mut DurableOpIndex,
    state: &'a mut GroupState,
    /// Op ids the STARTUP REPLAY quarantined (round-2 F1): a durable plan
    /// op in this set can never complete to approved - the divergence is
    /// durable, so the disposition must be too.
    replay_quarantined: &'a std::collections::BTreeSet<OpId>,
    now_ms: i64,
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
    let replay_quarantined: std::collections::BTreeSet<OpId> = fold_report
        .quarantined
        .iter()
        .map(|entry| entry.op_id)
        .collect();
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

    let scan = queue::scan(paths)?;
    warnings.extend(scan.io_warnings.iter().cloned());
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

    // Crash-state classification (cn-ingest owns the table).
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

    let mut recovery = Vec::new();
    let mut halts = Vec::new();
    let mut ops_appended = 0usize;

    // STOP-THE-LINE pre-pass (D-070.2 as amended, round-1 F5): every
    // halt-class row is reported and the run executes NOTHING.
    for (action, _) in &classified {
        match action {
            RecoveryAction::HaltLostDecisionState { record_id } => halts.push(format!(
                "record {record_id}: review-begun marker present without a readable \
                 sidecar (or without any files at all) - lost decision state; nothing \
                 is guessed (ADR-005 D4)"
            )),
            RecoveryAction::HaltBindingMismatch { record_id } => halts.push(format!(
                "record {record_id}: sidecar binding does not match its payload; both \
                 files retained for facilitator investigation, no automatic repair \
                 (ADR-005 D4)"
            )),
            _ => {}
        }
    }
    let halted_before_execution = !halts.is_empty();

    let mut live: BTreeMap<String, Live> = BTreeMap::new();
    if !halted_before_execution {
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
                            "marker present with unconsumed decisions; benign transient"
                                .to_string(),
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
                    let sidecar = ReviewSidecar::initial(&record)
                        .map_err(|ingest_err| ingest_err.to_string())?;
                    write_sidecar(paths, &sidecar)?;
                    recovery.push(RecoveryEntry {
                        record_id: record_id.clone(),
                        action: "reconstruct_pending_sidecar".to_string(),
                        detail: Some(
                            "initial pending state only; no history recreated".to_string(),
                        ),
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
                RecoveryAction::HaltLostDecisionState { .. }
                | RecoveryAction::HaltBindingMismatch { .. } => {
                    unreachable!("halt rows returned before execution")
                }
                RecoveryAction::RunApprovalRecovery { record_id } => {
                    let record = found
                        .payload
                        .ok_or_else(|| format!("record {record_id}: classify invariant broken"))?;
                    let FoundSidecar::Valid(sidecar) = found.sidecar else {
                        return Err(format!("record {record_id}: classify invariant broken"));
                    };
                    let mut sidecar = *sidecar;
                    let mut context = TxContext {
                        paths,
                        log: &mut log,
                        index: &mut index,
                        state: &mut state,
                        replay_quarantined: &replay_quarantined,
                        now_ms,
                    };
                    match recover_approval(
                        &mut context,
                        &record_id,
                        &record,
                        &template,
                        &mut sidecar,
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
                        Err(message) => {
                            // Stop-the-line: a failed approval recovery
                            // halts the run at this point; nothing further
                            // executes (D-070.2 as amended).
                            halts.push(message);
                            break;
                        }
                    }
                }
            }
        }
    }

    // Startup tombstone reconciliation (round-1 F8: digest-bound): a
    // consumed tombstone must match a decision event of its record by
    // decision_id AND message digest (I12; never silently deleted).
    let mut tombstone_anomalies = Vec::new();
    // After a pre-pass halt `live` is empty; reconciling against it would
    // report every tombstone as a false anomaly (round-2 F5 note).
    let reconcile_tombstones = !halted_before_execution;
    for (name, parsed) in scan.tombstones.iter().filter(|_| reconcile_tombstones) {
        match parsed {
            Err(reason) => {
                tombstone_anomalies.push(format!("unreadable tombstone '{name}': {reason}"));
            }
            Ok(message) => {
                let digest = message
                    .message_digest()
                    .map_err(|ingest_err| ingest_err.to_string())?;
                let recorded = live.get(&message.record_id).is_some_and(|entry| {
                    has_decision_event(&entry.sidecar, &message.decision_id, &digest)
                });
                if !recorded {
                    tombstone_anomalies.push(format!(
                        "tombstone '{name}': decision {} matches no decision event of \
                         record {} by id AND digest - impossible under the ordering \
                         rule; investigate (I12)",
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
            let mut context = TxContext {
                paths,
                log: &mut log,
                index: &mut index,
                state: &mut state,
                replay_quarantined: &replay_quarantined,
                now_ms,
            };
            match consume_decision(
                &mut context,
                &file,
                &message,
                &mut live,
                &template,
                &template_version,
                group_id,
                facilitator,
                &default_kind,
                &mut warnings,
            ) {
                Ok(Consumed::Entry(entry, appended)) => {
                    if matches!(
                        entry.outcome.as_str(),
                        "id_reuse_conflict"
                            | "binding_mismatch"
                            | "unknown_record"
                            | "reviewer_mismatch"
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
    context: &mut TxContext<'_>,
    file: &Path,
    message: &DecisionMessage,
    live: &mut BTreeMap<String, Live>,
    template: &cn_schema::GroupTemplate,
    template_version: &semver::Version,
    group_id: GroupId,
    facilitator: PersonId,
    default_kind: &KindId,
    warnings: &mut Vec<String>,
) -> Result<Consumed, String> {
    let entry = |outcome: &str, transaction: Option<TransactionEntry>| DecisionEntry {
        decision_id: message.decision_id.clone(),
        record_id: message.record_id.clone(),
        outcome: outcome.to_string(),
        transaction,
    };

    // Reviewer-identity binding (round-1 F2): the message's reviewer must
    // be the authorized person this apply run acts as - the ops'
    // responsible_human (ADR-005 D5) and the authority PermAuthorizer
    // checks. A mismatched or unparseable reviewer is never admitted
    // under borrowed authority; the file stays for facilitator
    // disposition (I3, I6).
    match message.reviewer.parse::<PersonId>() {
        Ok(reviewer) if reviewer == facilitator => {}
        _ => {
            warnings.push(format!(
                "decision {} names reviewer '{}' but this apply run is authorized as \
                 facilitator {}; refusing to admit under borrowed authority - file \
                 left for facilitator disposition (I6)",
                message.decision_id, message.reviewer, facilitator
            ));
            return Ok(Consumed::Entry(entry("reviewer_mismatch", None), 0));
        }
    }

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
            queue::retire_decision(context.paths, file)?;
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
            queue::ensure_marker(context.paths, &message.record_id)?;
            apply_verdict(&mut target.sidecar, &verdict)
                .map_err(|ingest_err| ingest_err.to_string())?;
            write_sidecar(context.paths, &target.sidecar)?;
            queue::retire_decision(context.paths, file)?;
            Ok(Consumed::Entry(entry(outcome, None), 0))
        }
        AdmissionVerdict::Admit { .. } => {
            queue::ensure_marker(context.paths, &message.record_id)?;
            let mut planned: Option<ApprovalPlan> = None;
            let mut submission_errors: Vec<String> = Vec::new();
            if matches!(message.decision, DecisionType::Approve) {
                let plan_context = PlanContext {
                    group_id,
                    facilitator,
                    kind: kind_for(&target.record, default_kind, warnings),
                    now_ms: context.now_ms,
                    template_version: template_version.clone(),
                };
                let plan = plan_approval(&target.record, template, &plan_context, &mut || {
                    uuid::Uuid::now_v7()
                })
                .map_err(|ingest_err| ingest_err.to_string())?;
                let submission = validate_submission(&target.record.payload, template);
                submission_errors = submission.errors.clone();
                target.sidecar.plan = Some(plan.plan_ref.clone());
                target.sidecar.validation_report = Some(
                    json!({ "submission": submission, "entity": plan.validation }).to_string(),
                );
                planned = Some(plan);
            }
            // Admission + history + plan + intent: ONE atomic sidecar
            // replace (apply_verdict seals over the plan set above).
            apply_verdict(&mut target.sidecar, &verdict)
                .map_err(|ingest_err| ingest_err.to_string())?;
            write_sidecar(context.paths, &target.sidecar)?;
            queue::retire_decision(context.paths, file)?;

            let mut transaction = None;
            let mut appended_count = 0;
            if let Some(plan) = planned {
                let mut validation_errors = submission_errors;
                validation_errors.extend(
                    plan.validation
                        .errors
                        .iter()
                        .map(|finding| format!("{finding:?}")),
                );
                let digests = plan.plan_ref.per_op_digests.clone();
                let (summary, appended) = execute_transaction(
                    context,
                    &mut target.sidecar,
                    &message.decision_id,
                    plan.ops,
                    &digests,
                    BatchMode::FirstAttempt,
                    &validation_errors,
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
/// (ADR-005 D4, round-1 F1): the persisted plan is cross-field verified
/// (round-1 F6); an ALL-ABSENT plan re-runs the FULL first-attempt step -
/// preflight and validation gate included; only a plan with durable
/// presence completes under the intent marker without re-authorization.
fn recover_approval(
    context: &mut TxContext<'_>,
    record_id: &str,
    record: &QueueRecord,
    template: &cn_schema::GroupTemplate,
    sidecar: &mut ReviewSidecar,
) -> Result<(TransactionEntry, usize), String> {
    let plan = sidecar.plan.clone().ok_or_else(|| {
        format!("record {record_id}: approved_intent with no persisted plan; halting (I3)")
    })?;
    let ops = verify_persisted_plan(&plan)
        .map_err(|ingest_err| format!("record {record_id}: {ingest_err}; halting (I3)"))?;
    let decision_id = admitting_decision_id(sidecar).ok_or_else(|| {
        format!(
            "record {record_id}: approved_intent with no admitting approve decision \
             event; halting"
        )
    })?;
    let any_present = ops.iter().any(|op| context.index.contains(&op.op_id));
    let (mode, validation_errors) = if any_present {
        // Durable presence: the intent marker authorizes completion of
        // what first-attempt preflight already authorized.
        (BatchMode::RecoveryUnderIntent, Vec::new())
    } else {
        // ALL absent: nothing was appended - "re-run step 2" (ADR-005
        // D4), preflight and validation included, under the SAME plan.
        let submission = validate_submission(&record.payload, template);
        let mut errors = submission.errors;
        let report = validate_record(record, template, &plan_kind(&ops)).map_err(|ingest_err| {
            format!("record {record_id}: authoritative validation failed ({ingest_err}); halting")
        })?;
        errors.extend(report.errors.iter().map(|finding| format!("{finding:?}")));
        (BatchMode::FirstAttempt, errors)
    };
    execute_transaction(
        context,
        sidecar,
        &decision_id,
        ops,
        &plan.per_op_digests,
        mode,
        &validation_errors,
    )
}

/// The planned entity kind, recovered from the plan's own ops (recovery
/// has no CLI kind context; the plan is the authority).
fn plan_kind(ops: &[Operation]) -> KindId {
    for op in ops {
        if let cn_store::OpKind::EntityCreate { entity } = &op.kind {
            return entity.kind.clone();
        }
    }
    KindId::new(cn_ingest::DEFAULT_PILOT_KIND).expect("valid const")
}

/// Pushes the plan through the durable seam and records the outcome as the
/// linked transaction event (ADR-005 D4 step 3). The validation gate
/// (round-1 F9) preflight-fails BEFORE the seam on first attempts; a
/// failed sidecar rewrite is a halt, not a warning. After a recovery
/// append, the real fold's quarantine report is checked - a quarantined
/// batch op is a loud halt, never a silent approval (round-1 F1 tail).
#[allow(clippy::too_many_arguments)]
fn execute_transaction(
    context: &mut TxContext<'_>,
    sidecar: &mut ReviewSidecar,
    decision_id: &str,
    ops: Vec<Operation>,
    digests: &[String],
    mode: BatchMode,
    validation_errors: &[String],
) -> Result<(TransactionEntry, usize), String> {
    let batch_op_ids: Vec<OpId> = ops.iter().map(|op| op.op_id).collect();
    // Sticky durable-quarantine rule (round-2 F1): if any planned op is
    // ALREADY durable and the startup replay quarantined it, the state
    // divergence is durable - record the terminal `durable_inconsistency`
    // disposition instead of ever reinterpreting an empty later scratch
    // report as success.
    let replayed_quarantined: Vec<String> = batch_op_ids
        .iter()
        .filter(|op_id| context.index.contains(op_id) && context.replay_quarantined.contains(op_id))
        .map(|op_id| op_id.to_string())
        .collect();
    if !replayed_quarantined.is_empty() {
        let detail = json!({
            "code": "durable_inconsistency",
            "quarantined_durable_ops": replayed_quarantined,
        })
        .to_string();
        record_transaction(
            sidecar,
            decision_id,
            TransactionKind::DurableInconsistency,
            ReviewState::Failed,
            Timestamp(context.now_ms),
            Some(detail.clone()),
        )
        .map_err(|ingest_err| ingest_err.to_string())?;
        write_sidecar(context.paths, sidecar)?;
        return Ok((
            TransactionEntry {
                kind: kind_name(TransactionKind::DurableInconsistency).to_string(),
                resulting_state: state_name(ReviewState::Failed).to_string(),
                detail: Some(detail),
            },
            0,
        ));
    }
    let outcome = if mode == BatchMode::FirstAttempt && !validation_errors.is_empty() {
        // Validation preflight (blueprint section 2 hostile-content rule):
        // an approve of a record with rejecting findings never reaches the
        // durable seam; the failure is recorded, loud, reviewable.
        Err(None)
    } else {
        let batch: Vec<BatchEntry> = ops
            .into_iter()
            .zip(digests.iter())
            .map(|(op, digest)| BatchEntry {
                op,
                digest: digest.clone(),
            })
            .collect();
        let mut scratch = StoreReport::default();
        let result = append_batch_idempotent(
            context.log,
            context.index,
            context.state,
            &PermAuthorizer,
            &batch,
            mode,
            &mut scratch,
        );
        if result.is_ok() {
            let quarantined: Vec<String> = scratch
                .quarantined
                .iter()
                .filter(|entry| batch_op_ids.contains(&entry.op_id))
                .map(|entry| entry.op_id.to_string())
                .collect();
            if !quarantined.is_empty() {
                // Durable terminal disposition, not a transient halt: the
                // ops are durable and quarantined, and a later run must
                // find `failed`, never reinterpret an all-present plan
                // with an empty scratch report as success (round-2 F1).
                let detail = json!({
                    "code": "durable_inconsistency",
                    "quarantined_durable_ops": quarantined,
                })
                .to_string();
                record_transaction(
                    sidecar,
                    decision_id,
                    TransactionKind::DurableInconsistency,
                    ReviewState::Failed,
                    Timestamp(context.now_ms),
                    Some(detail.clone()),
                )
                .map_err(|ingest_err| ingest_err.to_string())?;
                write_sidecar(context.paths, sidecar)?;
                return Ok((
                    TransactionEntry {
                        kind: kind_name(TransactionKind::DurableInconsistency).to_string(),
                        resulting_state: state_name(ReviewState::Failed).to_string(),
                        detail: Some(detail),
                    },
                    0,
                ));
            }
        }
        result.map_err(Some)
    };

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
        Err(None) => (
            TransactionKind::PreflightFailed,
            ReviewState::Pending,
            json!({ "code": "validation_failed", "errors": validation_errors }).to_string(),
            0,
        ),
        Err(Some(BatchFailure::Denied { op_id, denial })) => {
            require_first_attempt(mode, "authorization denial")?;
            (
                TransactionKind::PreflightFailed,
                ReviewState::Pending,
                json!({ "op_id": op_id.to_string(), "code": denial.code, "message": denial.message })
                    .to_string(),
                0,
            )
        }
        Err(Some(BatchFailure::WouldQuarantine { op_id })) => {
            require_first_attempt(mode, "would-quarantine")?;
            (
                TransactionKind::PreflightFailed,
                ReviewState::Pending,
                json!({ "op_id": op_id.to_string(), "code": "would_quarantine" }).to_string(),
                0,
            )
        }
        Err(Some(BatchFailure::DigestConflict { op_id })) => (
            TransactionKind::DurableConflict,
            ReviewState::Failed,
            json!({ "op_id": op_id.to_string(), "code": "durable_conflict" }).to_string(),
            0,
        ),
        Err(Some(BatchFailure::DurableInconsistency { op_id })) => (
            TransactionKind::DurableInconsistency,
            ReviewState::Failed,
            json!({ "op_id": op_id.to_string(), "code": "durable_inconsistency" }).to_string(),
            0,
        ),
        Err(Some(BatchFailure::PlanDigestMismatch { op_id })) => {
            return Err(format!(
                "plan digest mismatch for op {op_id}: the persisted plan is corrupt; \
                 halting (I3)"
            ));
        }
        Err(Some(BatchFailure::Store(store_err))) => {
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
        Timestamp(context.now_ms),
        Some(detail.clone()),
    )
    .map_err(|ingest_err| ingest_err.to_string())?;
    write_sidecar(context.paths, sidecar).map_err(|write_err| {
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

/// Digest-bound decision-event lookup (round-1 F8): id alone is not
/// identity - the recorded message digest must match too.
fn has_decision_event(sidecar: &ReviewSidecar, decision_id: &str, digest: &str) -> bool {
    sidecar.history.iter().any(|entry| {
        matches!(
            entry,
            HistoryEntry::Decision { decision_id: recorded, message_digest, .. }
                if recorded == decision_id && message_digest == digest
        )
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
