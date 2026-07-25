//! `cn intake apply` integration battery (blueprint section 9 step 5): the
//! full decide -> apply -> reload round trip on synthetic data, crash
//! replay, stale serialization, approval recovery under a durable intent,
//! and the authority-matrix boundary (preflight denial returns to pending).
//!
//! The test plays the app's create-only role by hand: it stages payload
//! records, initial sidecars, and decision files exactly as the FSA
//! adapter will (section 1 write contract), then invokes the router the
//! way the facilitator would.

use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use serde_json::{Value, json};

use cn_ingest::{
    DecisionMessage, DecisionType, PlanContext, QueueRecord, ReviewSidecar, ReviewState,
    SubmissionSource, admit, apply_verdict, decision_message_version, plan_approval,
};
use cn_model::{
    ActorRef, Group, GroupRole, Lifecycle, Membership, Origin, ProvenanceEnvelope, SensitivityTier,
    TemplateId, Timestamp,
};
use cn_store::{Hlc, OpKind, Operation, SortKey};

fn uuid_string(n: u128) -> String {
    let hex = format!("{n:032x}");
    format!(
        "{}-{}-{}-{}-{}",
        &hex[0..8],
        &hex[8..12],
        &hex[12..16],
        &hex[16..20],
        &hex[20..32]
    )
}

fn id<T>(n: u128) -> T
where
    T: FromStr,
    T::Err: std::fmt::Debug,
{
    T::from_str(&uuid_string(n)).expect("valid deterministic id")
}

fn group_uuid() -> String {
    uuid_string(10)
}

fn facilitator_uuid() -> String {
    uuid_string(2)
}

fn envelope(person: u128) -> ProvenanceEnvelope {
    ProvenanceEnvelope::new(
        Origin::Authored,
        ActorRef::Human(id(person)),
        id(person),
        Timestamp(1),
    )
    .expect("valid provenance")
}

fn template_json() -> String {
    r##"{
        "schema_version": "0.1.0",
        "template_id": "research",
        "name": "Research Network",
        "description": "Synthetic research network",
        "kinds": [{
            "id": "person",
            "label": "Person",
            "shape": "sphere",
            "color_role": "kind-1",
            "attributes": [
                { "id": "display_name", "type": "text", "required": true },
                { "id": "affiliation", "type": "tags" }
            ]
        }],
        "edge_kinds": [],
        "theme": { "mode": "light", "roles": { "kind-1": "#112233" } }
    }"##
    .to_string()
}

fn fixture_op(sort: i64, kind: OpKind) -> Operation {
    let actor = ActorRef::Human(id(1));
    let op_id = id(300 + sort as u128);
    Operation {
        op_id,
        group_id: id(10),
        actor: actor.clone(),
        responsible_human: id(1),
        recorded_at: Timestamp(sort),
        sort_key: SortKey::new(
            Hlc {
                wall_ms: sort,
                counter: 0,
            },
            &actor,
            op_id,
        ),
        template_version: semver::Version::new(0, 1, 0),
        kind,
        schema_version: cn_model::model_schema_version(),
    }
}

fn group_create_op() -> Operation {
    let group = Group::new(
        id(10),
        "Synthetic Group",
        TemplateId::new("research").expect("template id"),
        semver::Version::new(0, 1, 0),
        envelope(1),
        SensitivityTier::T1,
    )
    .expect("valid group");
    fixture_op(
        1,
        OpKind::GroupCreate {
            group,
            template_json: template_json(),
        },
    )
}

fn facilitator_membership_op() -> Operation {
    let mut membership =
        Membership::new(id(40), id(10), id(2), GroupRole::Facilitator, envelope(1));
    membership.lifecycle = Lifecycle::Active;
    fixture_op(2, OpKind::MembershipAdd { membership })
}

/// Writes the fixture ops log; `with_facilitator` controls whether the
/// facilitator holds an active membership (the authority-matrix lever).
fn write_ops_log(path: &Path, with_facilitator: bool) {
    let mut ops = vec![group_create_op()];
    if with_facilitator {
        ops.push(facilitator_membership_op());
    }
    let lines: Vec<String> = ops
        .iter()
        .map(|op| serde_json::to_string(op).expect("op serializes"))
        .collect();
    fs::write(path, lines.join("\n") + "\n").expect("ops log written");
}

fn payload() -> Value {
    json!({
        "submission_version": "0.1.0",
        "submission_id": "sub-1",
        "form_version": "form-0.1",
        "consent": {
            "consent_text_digest": "digest-consent",
            "consent_affirmed": true,
            "consent_affirmed_at": 5
        },
        "captured_at": 4,
        "fields": {
            "display_name": "Synthetic Person",
            "affiliation": ["River Alliance"]
        }
    })
}

fn staged_record(record_id: &str) -> QueueRecord {
    QueueRecord::new(
        record_id.to_string(),
        Timestamp(10),
        SubmissionSource::InApp {},
        payload(),
    )
    .expect("record builds")
}

/// Plays the app's create-only staging role: payload record + initial
/// pending sidecar.
fn stage_pair(queue: &Path, record: &QueueRecord) -> ReviewSidecar {
    fs::write(
        queue.join(format!("{}.record.json", record.record_id)),
        serde_json::to_vec_pretty(record).expect("record serializes"),
    )
    .expect("record staged");
    let sidecar = ReviewSidecar::initial(record).expect("sidecar builds");
    write_sidecar_file(queue, &sidecar);
    sidecar
}

fn write_sidecar_file(queue: &Path, sidecar: &ReviewSidecar) {
    fs::write(
        queue.join(format!("{}.sidecar.json", sidecar.record_id)),
        serde_json::to_vec_pretty(sidecar).expect("sidecar serializes"),
    )
    .expect("sidecar staged");
}

fn decision_message(
    record: &QueueRecord,
    decision_id: &str,
    decided_at: i64,
    expected_state: ReviewState,
    expected_generation: u64,
    decision: DecisionType,
) -> DecisionMessage {
    DecisionMessage {
        queue_record_version: decision_message_version(),
        decision_id: decision_id.to_string(),
        record_id: record.record_id.clone(),
        payload_digest: record.record_checksum.clone(),
        expected_review_state: expected_state,
        expected_decision_generation: expected_generation,
        decision,
        reviewer: facilitator_uuid(),
        decided_at: Timestamp(decided_at),
    }
}

/// Plays the app's create-only decision-file role.
fn drop_decision(queue: &Path, message: &DecisionMessage) -> PathBuf {
    let dir = queue.join("decisions");
    fs::create_dir_all(&dir).expect("decisions dir");
    let file = dir.join(format!(
        "{}.{}.json",
        message.record_id, message.decision_id
    ));
    fs::write(
        &file,
        serde_json::to_vec_pretty(message).expect("decision serializes"),
    )
    .expect("decision dropped");
    file
}

struct RunOutcome {
    exit: cn::Exit,
    report: Option<Value>,
    stderr: String,
}

fn run_apply(queue: &Path, ops: &Path) -> RunOutcome {
    let args: Vec<String> = [
        "intake",
        "apply",
        "--queue",
        queue.to_str().expect("queue path utf8"),
        "--ops",
        ops.to_str().expect("ops path utf8"),
        "--group",
        &group_uuid(),
        "--facilitator",
        &facilitator_uuid(),
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    let mut out = Vec::new();
    let mut err = Vec::new();
    let exit = cn::run(&args, &mut out, &mut err).expect("streams write");
    let stdout = String::from_utf8(out).expect("stdout utf8");
    RunOutcome {
        exit,
        report: serde_json::from_str(&stdout).ok(),
        stderr: String::from_utf8(err).expect("stderr utf8"),
    }
}

fn read_sidecar(queue: &Path, record_id: &str) -> ReviewSidecar {
    let bytes =
        fs::read(queue.join(format!("{record_id}.sidecar.json"))).expect("sidecar readable");
    serde_json::from_slice(&bytes).expect("sidecar parses")
}

fn ops_line_count(ops: &Path) -> usize {
    fs::read_to_string(ops)
        .expect("ops readable")
        .lines()
        .filter(|line| !line.trim().is_empty())
        .count()
}

fn setup() -> (tempfile::TempDir, PathBuf, PathBuf) {
    let dir = tempfile::tempdir().expect("tempdir");
    let queue = dir.path().join("queue");
    fs::create_dir_all(&queue).expect("queue dir");
    let ops = dir.path().join("ops.jsonl");
    (dir, queue, ops)
}

#[test]
fn refuses_queue_root_inside_a_git_worktree() {
    let dir = tempfile::tempdir().expect("tempdir");
    let repo = dir.path().join("repo");
    fs::create_dir_all(repo.join(".git")).expect("fake worktree");
    let queue = repo.join("queue");
    fs::create_dir_all(&queue).expect("queue dir");
    let ops = dir.path().join("ops.jsonl");
    write_ops_log(&ops, true);

    let outcome = run_apply(&queue, &ops);
    assert_eq!(outcome.exit, cn::Exit::Failure);
    assert!(
        outcome.stderr.contains("worktree"),
        "guard names the worktree: {}",
        outcome.stderr
    );
}

#[test]
fn approve_decide_apply_reload_round_trip() {
    let (_dir, queue, ops) = setup();
    write_ops_log(&ops, true);
    let record = staged_record(&uuid_string(0xE1));
    stage_pair(&queue, &record);
    drop_decision(
        &queue,
        &decision_message(
            &record,
            &uuid_string(0xD1),
            100,
            ReviewState::Pending,
            0,
            DecisionType::Approve,
        ),
    );

    let outcome = run_apply(&queue, &ops);
    assert_eq!(outcome.exit, cn::Exit::Ok, "stderr: {}", outcome.stderr);
    let report = outcome.report.expect("run report is JSON");
    assert_eq!(report["ops_appended"], 1, "one EntityCreate appended");
    assert_eq!(report["decisions"][0]["outcome"], "admitted");
    assert_eq!(
        report["decisions"][0]["transaction"]["kind"],
        "intent_completed"
    );
    assert_eq!(report["review_states"]["approved"], 1);

    // Sidecar: approved, decision event + transaction event, plan kept.
    let sidecar = read_sidecar(&queue, &record.record_id);
    assert_eq!(sidecar.review_state, ReviewState::Approved);
    assert_eq!(sidecar.decision_generation, 2, "admit + completion");
    assert_eq!(sidecar.history.len(), 2);
    assert!(sidecar.plan.is_some(), "plan persists for audit");
    assert!(sidecar.validation_report.is_some());

    // Files: marker present, decision retired to consumed/, log grew.
    assert!(
        queue
            .join(format!("{}.reviewed", record.record_id))
            .exists()
    );
    let consumed: Vec<_> = fs::read_dir(queue.join("decisions").join("consumed"))
        .expect("consumed dir")
        .collect();
    assert_eq!(consumed.len(), 1, "decision file retired as a tombstone");
    assert_eq!(ops_line_count(&ops), 3, "group + membership + entity");

    // Reload leg: the existing load path sees the approved entity.
    let ops_jsonl = fs::read_to_string(&ops).expect("ops readable");
    let viewer = json!({ "kind": "person", "person": facilitator_uuid() }).to_string();
    let mut api = cn_api::Api::new();
    let begin = api.load_group_begin(&group_uuid(), &viewer, &template_json());
    assert!(!begin.contains("\"err\""), "begin: {begin}");
    let chunk = api.load_ops_chunk(&group_uuid(), &ops_jsonl);
    assert!(!chunk.contains("\"err\""), "chunk: {chunk}");
    let commit = api.load_group_commit(&group_uuid(), 5_000);
    assert!(!commit.contains("\"err\""), "commit: {commit}");
    let snapshot = api.export_snapshot(&group_uuid(), &viewer, "{}");
    assert!(
        snapshot.contains("Synthetic Person"),
        "approved entity visible after reload"
    );

    // Idempotent second run: nothing to consume, nothing changes.
    let again = run_apply(&queue, &ops);
    assert_eq!(again.exit, cn::Exit::Ok, "stderr: {}", again.stderr);
    assert_eq!(ops_line_count(&ops), 3);
    let unchanged = read_sidecar(&queue, &record.record_id);
    assert_eq!(unchanged.sidecar_revision, sidecar.sidecar_revision);
}

#[test]
fn crash_replay_of_a_consumed_decision_is_writeless() {
    let (_dir, queue, ops) = setup();
    write_ops_log(&ops, true);
    let record = staged_record(&uuid_string(0xE2));
    stage_pair(&queue, &record);
    let decision = decision_message(
        &record,
        &uuid_string(0xD2),
        100,
        ReviewState::Pending,
        0,
        DecisionType::Approve,
    );
    let file = drop_decision(&queue, &decision);
    assert_eq!(run_apply(&queue, &ops).exit, cn::Exit::Ok);

    // Crash-before-retirement replay: the SAME bytes reappear in the inbox.
    let tombstone = queue
        .join("decisions")
        .join("consumed")
        .join(file.file_name().expect("name"));
    let before =
        fs::read(queue.join(format!("{}.sidecar.json", record.record_id))).expect("sidecar bytes");
    fs::copy(&tombstone, &file).expect("replayed decision file");

    let outcome = run_apply(&queue, &ops);
    assert_eq!(outcome.exit, cn::Exit::Ok, "stderr: {}", outcome.stderr);
    let report = outcome.report.expect("report");
    assert_eq!(report["decisions"][0]["outcome"], "writeless_replay");
    let after =
        fs::read(queue.join(format!("{}.sidecar.json", record.record_id))).expect("sidecar bytes");
    assert_eq!(before, after, "replay moves no counter and writes nothing");
    assert!(!file.exists(), "replay retired against the original event");
    assert_eq!(ops_line_count(&ops), 3, "no duplicate append");
}

#[test]
fn stale_decision_serializes_after_an_admitted_one() {
    let (_dir, queue, ops) = setup();
    write_ops_log(&ops, true);
    let record = staged_record(&uuid_string(0xE3));
    stage_pair(&queue, &record);
    // Both authored against generation 0; the reject decides first.
    drop_decision(
        &queue,
        &decision_message(
            &record,
            &uuid_string(0xD3),
            100,
            ReviewState::Pending,
            0,
            DecisionType::Reject {
                reason: "synthetic reject reason".to_string(),
            },
        ),
    );
    drop_decision(
        &queue,
        &decision_message(
            &record,
            &uuid_string(0xD4),
            200,
            ReviewState::Pending,
            0,
            DecisionType::Approve,
        ),
    );

    let outcome = run_apply(&queue, &ops);
    assert_eq!(outcome.exit, cn::Exit::Ok, "stderr: {}", outcome.stderr);
    let report = outcome.report.expect("report");
    assert_eq!(report["decisions"][0]["outcome"], "admitted");
    assert_eq!(report["decisions"][1]["outcome"], "stale");

    let sidecar = read_sidecar(&queue, &record.record_id);
    assert_eq!(sidecar.review_state, ReviewState::Rejected);
    assert_eq!(
        sidecar.decision_generation, 1,
        "stale audit entries never advance the generation"
    );
    assert_eq!(sidecar.history.len(), 2, "admitted + stale audit");
    assert_eq!(ops_line_count(&ops), 2, "no entity from the stale approve");
    let consumed = fs::read_dir(queue.join("decisions").join("consumed"))
        .expect("consumed dir")
        .count();
    assert_eq!(consumed, 2, "both decisions retired after durable events");
}

#[test]
fn recovery_completes_a_durable_approved_intent() {
    let (_dir, queue, ops) = setup();
    write_ops_log(&ops, true);
    let record = staged_record(&uuid_string(0xE4));

    // Build the approved_intent state exactly as a crashed first attempt
    // leaves it: admitted approve + plan in ONE sealed sidecar, marker
    // present, nothing appended to the durable log yet.
    let (template, report) = cn_schema::parse_template(&template_json()).expect("template");
    assert!(report.errors.is_empty());
    let context = PlanContext {
        group_id: id(10),
        facilitator: id(2),
        kind: cn_model::KindId::new("person").expect("kind"),
        now_ms: 1_000,
        template_version: semver::Version::new(0, 1, 0),
    };
    let mut n: u128 = 0;
    let plan = plan_approval(&record, &template, &context, &mut || {
        n += 1;
        uuid::Uuid::from_u128(0xABCD_0000_0000_0000_0000_0000_0000_0000 + n)
    })
    .expect("plan builds");
    let mut sidecar = ReviewSidecar::initial(&record).expect("sidecar");
    let message = decision_message(
        &record,
        &uuid_string(0xD5),
        100,
        ReviewState::Pending,
        0,
        DecisionType::Approve,
    );
    let verdict = admit(&sidecar, &message).expect("verdict");
    sidecar.plan = Some(plan.plan_ref.clone());
    apply_verdict(&mut sidecar, &verdict).expect("admitted");
    assert_eq!(sidecar.review_state, ReviewState::ApprovedIntent);
    fs::write(
        queue.join(format!("{}.record.json", record.record_id)),
        serde_json::to_vec_pretty(&record).expect("record serializes"),
    )
    .expect("record staged");
    write_sidecar_file(&queue, &sidecar);
    fs::write(queue.join(format!("{}.reviewed", record.record_id)), b"").expect("marker");

    let outcome = run_apply(&queue, &ops);
    assert_eq!(outcome.exit, cn::Exit::Ok, "stderr: {}", outcome.stderr);
    let report = outcome.report.expect("report");
    assert_eq!(report["recovery"][0]["action"], "approval_recovery");
    assert_eq!(report["ops_appended"], 1, "absent plan ops appended once");

    let recovered = read_sidecar(&queue, &record.record_id);
    assert_eq!(recovered.review_state, ReviewState::Approved);
    assert_eq!(recovered.decision_generation, 2, "admit + completion");
    assert_eq!(ops_line_count(&ops), 3, "entity durable exactly once");

    // A second run finds nothing to recover and appends nothing.
    let again = run_apply(&queue, &ops);
    assert_eq!(again.exit, cn::Exit::Ok);
    assert_eq!(ops_line_count(&ops), 3);
}

#[test]
fn preflight_denial_returns_the_record_to_pending() {
    let (_dir, queue, ops) = setup();
    // No facilitator membership: the authority matrix denies the unowned
    // create (not_member), so the transaction preflight-fails.
    write_ops_log(&ops, false);
    let record = staged_record(&uuid_string(0xE5));
    stage_pair(&queue, &record);
    drop_decision(
        &queue,
        &decision_message(
            &record,
            &uuid_string(0xD6),
            100,
            ReviewState::Pending,
            0,
            DecisionType::Approve,
        ),
    );

    let outcome = run_apply(&queue, &ops);
    assert_eq!(outcome.exit, cn::Exit::Ok, "stderr: {}", outcome.stderr);
    let report = outcome.report.expect("report");
    assert_eq!(report["decisions"][0]["outcome"], "admitted");
    assert_eq!(
        report["decisions"][0]["transaction"]["kind"],
        "preflight_failed"
    );
    assert!(
        report["decisions"][0]["transaction"]["detail"]
            .as_str()
            .expect("detail")
            .contains("not_member"),
        "the denial code is recorded"
    );

    let sidecar = read_sidecar(&queue, &record.record_id);
    assert_eq!(
        sidecar.review_state,
        ReviewState::Pending,
        "preflight failure returns to pending - loud, reviewable, no partial state"
    );
    assert_eq!(sidecar.decision_generation, 2, "admit + preflight_failed");
    assert_eq!(ops_line_count(&ops), 1, "nothing appended");
}

// --- round-1 amendments (2026-07-25 adversarial round) ---

/// F1: a crash after a first-attempt DENIAL leaves a durable
/// approved_intent with an all-absent plan. Recovery must re-run the
/// FULL first-attempt step (ADR-005 D4 "re-run step 2") - the denial
/// recurs, the record returns to pending, and nothing is ever appended
/// without authorization.
#[test]
fn crash_after_denial_recovers_with_full_preflight() {
    let (_dir, queue, ops) = setup();
    // No facilitator membership: authorization DENIES the unowned create.
    write_ops_log(&ops, false);
    let record = staged_record(&uuid_string(0xE6));

    let (template, report) = cn_schema::parse_template(&template_json()).expect("template");
    assert!(report.errors.is_empty());
    let context = PlanContext {
        group_id: id(10),
        facilitator: id(2),
        kind: cn_model::KindId::new("person").expect("kind"),
        now_ms: 1_000,
        template_version: semver::Version::new(0, 1, 0),
    };
    let mut n: u128 = 0;
    let plan = plan_approval(&record, &template, &context, &mut || {
        n += 1;
        uuid::Uuid::from_u128(0xBEEF_0000_0000_0000_0000_0000_0000_0000 + n)
    })
    .expect("plan builds");
    let mut sidecar = ReviewSidecar::initial(&record).expect("sidecar");
    let message = decision_message(
        &record,
        &uuid_string(0xD7),
        100,
        ReviewState::Pending,
        0,
        DecisionType::Approve,
    );
    let verdict = admit(&sidecar, &message).expect("verdict");
    sidecar.plan = Some(plan.plan_ref.clone());
    apply_verdict(&mut sidecar, &verdict).expect("admitted");
    assert_eq!(sidecar.review_state, ReviewState::ApprovedIntent);
    fs::write(
        queue.join(format!("{}.record.json", record.record_id)),
        serde_json::to_vec_pretty(&record).expect("record serializes"),
    )
    .expect("record staged");
    write_sidecar_file(&queue, &sidecar);
    fs::write(queue.join(format!("{}.reviewed", record.record_id)), b"").expect("marker");

    let outcome = run_apply(&queue, &ops);
    assert_eq!(outcome.exit, cn::Exit::Ok, "stderr: {}", outcome.stderr);
    let report = outcome.report.expect("report");
    assert_eq!(report["recovery"][0]["action"], "approval_recovery");
    assert!(
        report["recovery"][0]["detail"]
            .as_str()
            .expect("detail")
            .contains("preflight_failed"),
        "denial recurs at recovery: {report}"
    );

    let recovered = read_sidecar(&queue, &record.record_id);
    assert_eq!(
        recovered.review_state,
        ReviewState::Pending,
        "denied plan NEVER completes under the intent marker"
    );
    assert_eq!(
        ops_line_count(&ops),
        1,
        "nothing appended without authorization"
    );
}

/// F2: a decision naming a different reviewer than the authorized
/// facilitator is refused - authority is never borrowed from the CLI
/// argument.
#[test]
fn reviewer_mismatch_is_refused() {
    let (_dir, queue, ops) = setup();
    write_ops_log(&ops, true);
    let record = staged_record(&uuid_string(0xE7));
    let sidecar_before = stage_pair(&queue, &record);
    let mut message = decision_message(
        &record,
        &uuid_string(0xD8),
        100,
        ReviewState::Pending,
        0,
        DecisionType::Approve,
    );
    message.reviewer = uuid_string(0x99); // NOT the --facilitator person
    let file = drop_decision(&queue, &message);

    let outcome = run_apply(&queue, &ops);
    assert_eq!(outcome.exit, cn::Exit::Failure, "attention required");
    let report = outcome.report.expect("report");
    assert_eq!(report["decisions"][0]["outcome"], "reviewer_mismatch");
    assert!(file.exists(), "file left for facilitator disposition");
    let unchanged = read_sidecar(&queue, &record.record_id);
    assert_eq!(
        unchanged.sidecar_revision, sidecar_before.sidecar_revision,
        "nothing admitted under borrowed authority"
    );
    assert_eq!(ops_line_count(&ops), 2, "nothing appended");
}

/// F8: tombstone reconciliation is digest-bound - a tombstone whose id
/// matches a decision event but whose BYTES differ is an anomaly.
#[test]
fn tampered_tombstone_is_a_reconciliation_anomaly() {
    let (_dir, queue, ops) = setup();
    write_ops_log(&ops, true);
    let record = staged_record(&uuid_string(0xE8));
    stage_pair(&queue, &record);
    let message = decision_message(
        &record,
        &uuid_string(0xD9),
        100,
        ReviewState::Pending,
        0,
        DecisionType::Approve,
    );
    let file = drop_decision(&queue, &message);
    assert_eq!(run_apply(&queue, &ops).exit, cn::Exit::Ok);

    // Rewrite the tombstone with the SAME decision_id but different bytes.
    let tombstone = queue
        .join("decisions")
        .join("consumed")
        .join(file.file_name().expect("name"));
    let mut tampered = message.clone();
    tampered.decided_at = cn_model::Timestamp(999);
    fs::write(
        &tombstone,
        serde_json::to_vec_pretty(&tampered).expect("serializes"),
    )
    .expect("tampered tombstone written");

    let outcome = run_apply(&queue, &ops);
    assert_eq!(outcome.exit, cn::Exit::Failure);
    let report = outcome.report.expect("report");
    assert!(
        report["tombstone_anomalies"]
            .as_array()
            .expect("anomalies")
            .iter()
            .any(|entry| entry.as_str().expect("text").contains("digest")),
        "digest-bound reconciliation flags the tampered tombstone: {report}"
    );
}

/// F9: an approve of a payload whose consent is NOT affirmed
/// preflight-fails on the validation gate before the durable seam.
#[test]
fn unconsented_payload_preflight_fails_validation() {
    let (_dir, queue, ops) = setup();
    write_ops_log(&ops, true);
    let mut unconsented = payload();
    unconsented["consent"]["consent_affirmed"] = serde_json::json!(false);
    let record = QueueRecord::new(
        uuid_string(0xE9),
        Timestamp(10),
        SubmissionSource::InApp {},
        unconsented,
    )
    .expect("record builds");
    stage_pair(&queue, &record);
    drop_decision(
        &queue,
        &decision_message(
            &record,
            &uuid_string(0xDA),
            100,
            ReviewState::Pending,
            0,
            DecisionType::Approve,
        ),
    );

    let outcome = run_apply(&queue, &ops);
    assert_eq!(outcome.exit, cn::Exit::Ok, "stderr: {}", outcome.stderr);
    let report = outcome.report.expect("report");
    assert_eq!(
        report["decisions"][0]["transaction"]["kind"],
        "preflight_failed"
    );
    assert!(
        report["decisions"][0]["transaction"]["detail"]
            .as_str()
            .expect("detail")
            .contains("validation_failed"),
        "the validation gate is recorded: {report}"
    );
    let sidecar = read_sidecar(&queue, &record.record_id);
    assert_eq!(sidecar.review_state, ReviewState::Pending);
    assert_eq!(ops_line_count(&ops), 2, "nothing reached the durable seam");
}

/// Round-2 F1: a durable-but-quarantined plan op is a STICKY terminal
/// `failed` (durable_inconsistency) - a later run must never reinterpret
/// the all-present plan with an empty scratch report as success.
#[test]
fn quarantined_durable_plan_is_sticky_failed_across_runs() {
    let (_dir, queue, ops) = setup();
    write_ops_log(&ops, true);

    // A record whose entity would QUARANTINE at fold (missing required
    // display_name), with its op already durable - the round-2 timeline.
    let mut bare = payload();
    bare["fields"] = serde_json::json!({});
    let record = QueueRecord::new(
        uuid_string(0xEA),
        Timestamp(10),
        SubmissionSource::InApp {},
        bare,
    )
    .expect("record builds");
    let (template, report) = cn_schema::parse_template(&template_json()).expect("template");
    assert!(report.errors.is_empty());
    let context = PlanContext {
        group_id: id(10),
        facilitator: id(2),
        kind: cn_model::KindId::new("person").expect("kind"),
        now_ms: 1_000,
        template_version: semver::Version::new(0, 1, 0),
    };
    let mut n: u128 = 0;
    let plan = plan_approval(&record, &template, &context, &mut || {
        n += 1;
        uuid::Uuid::from_u128(0xFACE_0000_0000_0000_0000_0000_0000_0000 + n)
    })
    .expect("plan builds");

    // Append the planned op to the durable log (it will quarantine at
    // replay because the entity lacks its required attribute).
    let mut log_lines = fs::read_to_string(&ops).expect("ops readable");
    for op_value in &plan.plan_ref.ops {
        log_lines.push_str(&op_value.to_string());
        log_lines.push('\n');
    }
    fs::write(&ops, log_lines).expect("ops appended");

    let mut sidecar = ReviewSidecar::initial(&record).expect("sidecar");
    let message = decision_message(
        &record,
        &uuid_string(0xDB),
        100,
        ReviewState::Pending,
        0,
        DecisionType::Approve,
    );
    let verdict = admit(&sidecar, &message).expect("verdict");
    sidecar.plan = Some(plan.plan_ref.clone());
    apply_verdict(&mut sidecar, &verdict).expect("admitted");
    fs::write(
        queue.join(format!("{}.record.json", record.record_id)),
        serde_json::to_vec_pretty(&record).expect("record serializes"),
    )
    .expect("record staged");
    write_sidecar_file(&queue, &sidecar);
    fs::write(queue.join(format!("{}.reviewed", record.record_id)), b"").expect("marker");

    // Run 1: the durable-quarantine divergence lands as terminal failed.
    let first = run_apply(&queue, &ops);
    let sidecar_after_one = read_sidecar(&queue, &record.record_id);
    assert_eq!(
        sidecar_after_one.review_state,
        ReviewState::Failed,
        "durable quarantined plan op -> terminal failed: {:?}",
        first.report
    );
    let lines_after_one = ops_line_count(&ops);

    // Run 2: STICKY - still failed, nothing appended, never approved.
    let second = run_apply(&queue, &ops);
    let sidecar_after_two = read_sidecar(&queue, &record.record_id);
    assert_eq!(
        sidecar_after_two.review_state,
        ReviewState::Failed,
        "the next run never reinterprets the state as success: {:?}",
        second.report
    );
    assert_eq!(
        sidecar_after_two.sidecar_revision, sidecar_after_one.sidecar_revision,
        "terminal state is stable"
    );
    assert_eq!(ops_line_count(&ops), lines_after_one, "no further appends");
}
