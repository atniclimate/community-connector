//! ADR-005 D4 idempotent durable batch append: crash-simulation matrix.

use std::path::{Path, PathBuf};
use std::str::FromStr;

use cn_model::*;
use cn_store::*;
use semver::Version;

fn ts(ms: i64) -> Timestamp {
    Timestamp(ms)
}

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

fn person(n: u128) -> PersonId {
    id(n)
}

fn group_id() -> GroupId {
    id(10)
}

fn entity_id(n: u128) -> EntityId {
    id(100 + n)
}

fn op_id(n: u128) -> OpId {
    id(300 + n)
}

fn envelope() -> ProvenanceEnvelope {
    ProvenanceEnvelope::new(
        Origin::Authored,
        ActorRef::Human(person(1)),
        person(1),
        ts(1),
    )
    .expect("valid provenance")
}

fn version() -> Version {
    Version::new(0, 1, 0)
}

fn template_json() -> String {
    r##"{
        "schema_version": "0.1.0",
        "template_id": "research",
        "name": "Research Network",
        "description": "Synthetic research network",
        "kinds": [
            {
                "id": "person",
                "label": "Person",
                "shape": "sphere",
                "color_role": "kind-1",
                "attributes": [
                    { "id": "name", "type": "text" }
                ]
            }
        ],
        "edge_kinds": [
            {
                "id": "knows",
                "label": "Knows",
                "from": ["person"],
                "to": ["person"],
                "directed": false,
                "weighted": "forbidden"
            }
        ],
        "theme": {
            "mode": "light",
            "roles": { "kind-1": "#112233" }
        }
    }"##
    .to_string()
}

fn group_create(sort: i64) -> Operation {
    let group = Group::new(
        group_id(),
        "Synthetic Group",
        TemplateId::new("research").expect("template id"),
        version(),
        envelope(),
        SensitivityTier::T1,
    )
    .expect("valid group");
    op(
        sort,
        OpKind::GroupCreate {
            group,
            template_json: template_json(),
        },
    )
}

fn op(sort: i64, kind: OpKind) -> Operation {
    let actor = ActorRef::Human(person(1));
    let op_id = op_id(sort as u128);
    Operation {
        op_id,
        group_id: group_id(),
        actor: actor.clone(),
        responsible_human: person(1),
        recorded_at: ts(sort),
        sort_key: SortKey::new(
            Hlc {
                wall_ms: sort,
                counter: 0,
            },
            &actor,
            op_id,
        ),
        template_version: version(),
        kind,
        schema_version: cn_model::model_schema_version(),
    }
}

fn entity(n: u128) -> Entity {
    Entity::new(
        entity_id(n),
        group_id(),
        KindId::new("person").expect("kind id"),
        Circle::Group,
        envelope(),
        SensitivityTier::T1,
    )
}

struct AllowAll;

impl Authorizer for AllowAll {
    fn authorize(&self, _state: &GroupState, _op: &Operation) -> Result<(), AuthzDenial> {
        Ok(())
    }
}

struct DenyAll;

impl Authorizer for DenyAll {
    fn authorize(&self, _state: &GroupState, _op: &Operation) -> Result<(), AuthzDenial> {
        Err(AuthzDenial {
            code: "test_denied".to_string(),
            message: "denied by test authorizer".to_string(),
        })
    }
}

fn temp_path(name: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!("cn-store-seam-{name}-{}.jsonl", std::process::id()));
    let _ = std::fs::remove_file(&path);
    path
}

fn entries(ops: &[Operation]) -> Vec<BatchEntry> {
    ops.iter()
        .map(|op| BatchEntry {
            op: op.clone(),
            digest: canonical_op_digest(op).expect("digest"),
        })
        .collect()
}

fn log_line_count(path: &Path) -> usize {
    let bytes = std::fs::read(path).expect("read log");
    bytes.iter().filter(|byte| **byte == b'\n').count()
}

fn open_all(path: &Path) -> (OpLog, DurableOpIndex, GroupState, StoreReport) {
    let (log, ops, _) = OpLog::open(path).expect("open");
    let index = DurableOpIndex::from_ops(&ops).expect("index");
    let (state, report) = fold(ops);
    (log, index, state, report)
}

#[test]
fn first_attempt_appends_all_and_folds() {
    let path = temp_path("happy");
    let batch_ops = vec![
        group_create(1),
        op(2, OpKind::EntityCreate { entity: entity(1) }),
    ];
    let (mut log, mut index, mut state, mut report) = open_all(&path);
    let result = append_batch_idempotent(
        &mut log,
        &mut index,
        &mut state,
        &AllowAll,
        &entries(&batch_ops),
        BatchMode::FirstAttempt,
        &mut report,
    )
    .expect("batch succeeds");
    assert!(
        result
            .iter()
            .all(|(_, d)| *d == OpDisposition::AbsentAppended)
    );
    assert_eq!(log_line_count(&path), 2);
    assert_eq!(index.len(), 2);
    assert!(state.entities.contains_key(&entity_id(1)));
}

#[test]
fn crash_replay_appends_zero_duplicate_lines() {
    let path = temp_path("replay");
    let batch_ops = vec![
        group_create(1),
        op(2, OpKind::EntityCreate { entity: entity(1) }),
    ];
    {
        let (mut log, mut index, mut state, mut report) = open_all(&path);
        append_batch_idempotent(
            &mut log,
            &mut index,
            &mut state,
            &AllowAll,
            &entries(&batch_ops),
            BatchMode::FirstAttempt,
            &mut report,
        )
        .expect("first attempt");
    }
    // Crash before sidecar completion: recovery re-runs the SAME plan.
    let (mut log, mut index, mut state, mut report) = open_all(&path);
    let result = append_batch_idempotent(
        &mut log,
        &mut index,
        &mut state,
        &AllowAll,
        &entries(&batch_ops),
        BatchMode::RecoveryUnderIntent,
        &mut report,
    )
    .expect("recovery succeeds");
    assert!(
        result
            .iter()
            .all(|(_, d)| *d == OpDisposition::PresentSameDigest)
    );
    assert_eq!(log_line_count(&path), 2, "zero duplicate log lines");
}

#[test]
fn contiguous_prefix_appends_exactly_the_absent_suffix_without_reauthorization() {
    let path = temp_path("prefix");
    let batch_ops = vec![
        group_create(1),
        op(2, OpKind::EntityCreate { entity: entity(1) }),
    ];
    {
        // Simulate a crash after the first op became durable.
        let (mut log, mut index, mut state, mut report) = open_all(&path);
        append_batch_idempotent(
            &mut log,
            &mut index,
            &mut state,
            &AllowAll,
            &entries(&batch_ops[..1]),
            BatchMode::FirstAttempt,
            &mut report,
        )
        .expect("prefix");
    }
    // Recovery with DenyAll proves no re-authorization happens: the
    // intent marker governs (ADR-005 D4).
    let (mut log, mut index, mut state, mut report) = open_all(&path);
    let result = append_batch_idempotent(
        &mut log,
        &mut index,
        &mut state,
        &DenyAll,
        &entries(&batch_ops),
        BatchMode::RecoveryUnderIntent,
        &mut report,
    )
    .expect("recovery completes under the marker");
    assert_eq!(
        result,
        vec![
            (batch_ops[0].op_id, OpDisposition::PresentSameDigest),
            (batch_ops[1].op_id, OpDisposition::AbsentAppended),
        ]
    );
    assert_eq!(log_line_count(&path), 2);
    assert!(state.entities.contains_key(&entity_id(1)));
}

#[test]
fn hole_or_out_of_order_presence_fails_with_nothing_appended() {
    let path = temp_path("hole");
    let batch_ops = vec![
        group_create(1),
        op(2, OpKind::EntityCreate { entity: entity(1) }),
    ];
    {
        // Durable log holds ONLY the second op: an impossible hole for
        // this plan (sequential append cannot produce it).
        let (mut log, _, _, _) = open_all(&path);
        log.append_batch(&batch_ops[1..]).expect("seed hole");
    }
    let (mut log, mut index, mut state, mut report) = open_all(&path);
    let result = append_batch_idempotent(
        &mut log,
        &mut index,
        &mut state,
        &AllowAll,
        &entries(&batch_ops),
        BatchMode::RecoveryUnderIntent,
        &mut report,
    );
    assert_eq!(
        result,
        Err(BatchFailure::DurableInconsistency {
            op_id: batch_ops[1].op_id
        })
    );
    assert_eq!(log_line_count(&path), 1, "nothing appended");
}

#[test]
fn conflicting_digest_halts_with_nothing_appended() {
    let path = temp_path("conflict");
    let original = group_create(1);
    {
        let (mut log, _, _, _) = open_all(&path);
        log.append_batch(std::slice::from_ref(&original))
            .expect("seed");
    }
    // Same op id, different bytes.
    let mut tampered = original.clone();
    tampered.recorded_at = ts(999);
    let (mut log, mut index, mut state, mut report) = open_all(&path);
    let result = append_batch_idempotent(
        &mut log,
        &mut index,
        &mut state,
        &AllowAll,
        &entries(&[tampered]),
        BatchMode::RecoveryUnderIntent,
        &mut report,
    );
    assert_eq!(
        result,
        Err(BatchFailure::DigestConflict {
            op_id: original.op_id
        })
    );
    assert_eq!(log_line_count(&path), 1, "nothing appended");
}

#[test]
fn preflight_denial_fails_batch_with_nothing_appended() {
    let path = temp_path("denied");
    let batch_ops = vec![group_create(1)];
    let (mut log, mut index, mut state, mut report) = open_all(&path);
    let result = append_batch_idempotent(
        &mut log,
        &mut index,
        &mut state,
        &DenyAll,
        &entries(&batch_ops),
        BatchMode::FirstAttempt,
        &mut report,
    );
    assert!(matches!(result, Err(BatchFailure::Denied { .. })));
    assert_eq!(log_line_count(&path), 0, "nothing appended");
    assert!(index.is_empty());
}

#[test]
fn preflight_quarantine_fails_whole_batch_with_nothing_appended() {
    let path = temp_path("quarantine");
    // EntityCreate without any GroupCreate: the fold cannot accept it.
    let batch_ops = vec![op(2, OpKind::EntityCreate { entity: entity(1) })];
    let (mut log, mut index, mut state, mut report) = open_all(&path);
    let result = append_batch_idempotent(
        &mut log,
        &mut index,
        &mut state,
        &AllowAll,
        &entries(&batch_ops),
        BatchMode::FirstAttempt,
        &mut report,
    );
    assert!(
        matches!(result, Err(BatchFailure::WouldQuarantine { .. })),
        "got {result:?}"
    );
    assert_eq!(log_line_count(&path), 0, "nothing appended");
    assert!(state.entities.is_empty(), "state untouched");
}

#[test]
fn plan_digest_mismatch_fails_before_any_work() {
    let path = temp_path("plan");
    let batch_ops = vec![group_create(1)];
    let mut batch = entries(&batch_ops);
    batch[0].digest = "not-the-digest".to_string();
    let (mut log, mut index, mut state, mut report) = open_all(&path);
    let result = append_batch_idempotent(
        &mut log,
        &mut index,
        &mut state,
        &AllowAll,
        &batch,
        BatchMode::FirstAttempt,
        &mut report,
    );
    assert_eq!(
        result,
        Err(BatchFailure::PlanDigestMismatch {
            op_id: batch_ops[0].op_id
        })
    );
    assert_eq!(log_line_count(&path), 0);
}
