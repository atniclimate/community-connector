use std::path::PathBuf;
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

fn edge_id(n: u128) -> EdgeId {
    id(200 + n)
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
                    { "id": "name", "type": "text" },
                    { "id": "score", "type": "number" }
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

fn named_entity(n: u128, name: &str) -> Entity {
    let mut entity = entity(n);
    entity
        .attributes
        .insert(AttrId::new("name").expect("attr id"), attr_text(name));
    entity
}

fn named_entity_with_sort(n: u128, name: &str, sort: i64) -> Operation {
    op(
        sort,
        OpKind::AttributeSet {
            entity: entity_id(n),
            attr: AttrId::new("name").expect("attr id"),
            instance: attr_text(name),
        },
    )
}

fn attr_text(value: &str) -> AttributeInstance {
    AttributeInstance::new(
        AttributeValue::Text(value.to_string()),
        Circle::Group,
        envelope(),
    )
}

fn edge(n: u128, from: u128, to: u128) -> Edge {
    Edge::new(
        edge_id(n),
        group_id(),
        KindId::new("knows").expect("kind id"),
        entity_id(from),
        entity_id(to),
        false,
        Circle::Group,
        envelope(),
        SensitivityTier::T1,
    )
}

#[derive(Debug, PartialEq)]
struct PublicState {
    group: Option<Group>,
    entities: Vec<Entity>,
    edges: Vec<Edge>,
    memberships: Vec<Membership>,
    trust_grants: Vec<TrustGrant>,
    stories: Vec<Story>,
}

fn public_state(state: &GroupState) -> PublicState {
    PublicState {
        group: state.group.clone(),
        entities: state.entities.values().cloned().collect(),
        edges: state.edges.values().cloned().collect(),
        memberships: state.memberships.values().cloned().collect(),
        trust_grants: state.trust_grants.values().cloned().collect(),
        stories: state.stories.values().cloned().collect(),
    }
}

fn shuffled(mut ops: Vec<Operation>, seed: u64) -> Vec<Operation> {
    let mut state = seed;
    for i in (1..ops.len()).rev() {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
        let j = (state as usize) % (i + 1);
        ops.swap(i, j);
    }
    ops
}

#[test]
fn convergence_over_reordering_and_duplication() {
    let ops = vec![
        group_create(1),
        op(
            2,
            OpKind::EntityCreate {
                entity: named_entity(1, "A"),
            },
        ),
        op(
            3,
            OpKind::EntityCreate {
                entity: named_entity(2, "B"),
            },
        ),
        op(
            4,
            OpKind::EdgeCreate {
                edge: edge(1, 1, 2),
            },
        ),
        op(
            5,
            OpKind::AttributeSet {
                entity: entity_id(1),
                attr: AttrId::new("score").expect("attr id"),
                instance: AttributeInstance::new(
                    AttributeValue::Number(7.0),
                    Circle::Group,
                    envelope(),
                ),
            },
        ),
    ];
    let (base, _) = fold(ops.clone());
    for seed in [11, 22, 33] {
        let (shuffled, report) = fold(shuffled(ops.clone(), seed));
        assert_eq!(public_state(&base), public_state(&shuffled));
        assert!(report.quarantined.is_empty());
    }
    let mut duplicated = ops.clone();
    duplicated.extend(ops);
    let (duped, report) = fold(duplicated);
    assert_eq!(public_state(&base), public_state(&duped));
    assert!(report.deduped > 0);
}

#[test]
fn low_sort_admitted_late_does_not_overwrite_higher_lww() {
    let attr = AttrId::new("name").expect("attr id");
    let ops = vec![
        group_create(0),
        op(
            1,
            OpKind::AttributeSet {
                entity: entity_id(1),
                attr: attr.clone(),
                instance: attr_text("low"),
            },
        ),
        op(2, OpKind::EntityCreate { entity: entity(1) }),
        op(
            3,
            OpKind::AttributeSet {
                entity: entity_id(1),
                attr: attr.clone(),
                instance: attr_text("high"),
            },
        ),
    ];
    let (state, report) = fold(ops);
    assert!(report.quarantined.is_empty());
    assert_eq!(
        state.entities[&entity_id(1)].attributes[&attr].value,
        AttributeValue::Text("high".to_string())
    );
}

#[test]
fn fixpoint_admits_edges_after_deep_dependencies() {
    let story = Story::new(
        id(500),
        group_id(),
        "Story",
        vec![StoryStep {
            entity: entity_id(1),
            narration: "Synthetic step".to_string(),
        }],
        Circle::Group,
        envelope(),
        SensitivityTier::T1,
    );
    let ops = vec![
        group_create(0),
        op(1, OpKind::StoryCreate { story }),
        op(
            2,
            OpKind::EdgeCreate {
                edge: edge(1, 1, 2),
            },
        ),
        op(3, OpKind::EntityCreate { entity: entity(1) }),
        op(4, OpKind::EntityCreate { entity: entity(2) }),
    ];
    let (state, report) = fold(ops);
    assert!(report.quarantined.is_empty());
    assert_eq!(state.edges.len(), 1);
    assert_eq!(state.stories.len(), 1);
}

#[test]
fn lifecycle_lww_is_independent_from_attribute_writes() {
    let attr = AttrId::new("name").expect("attr id");
    let ops = vec![
        group_create(0),
        op(1, OpKind::EntityCreate { entity: entity(1) }),
        op(
            2,
            OpKind::EntityLifecycleSet {
                entity: entity_id(1),
                lifecycle: Lifecycle::Archived,
            },
        ),
        op(
            3,
            OpKind::AttributeSet {
                entity: entity_id(1),
                attr: attr.clone(),
                instance: attr_text("Archived Attr"),
            },
        ),
        op(
            4,
            OpKind::EntityLifecycleSet {
                entity: entity_id(1),
                lifecycle: Lifecycle::Active,
            },
        ),
    ];
    let (state, report) = fold(ops);
    assert!(report.quarantined.is_empty());
    let entity = &state.entities[&entity_id(1)];
    assert_eq!(entity.lifecycle, Lifecycle::Active);
    assert!(entity.attributes.contains_key(&attr));
}

#[test]
fn hlc_clock_monotonic_under_stall_and_regression() {
    let mut clock = HlcClock::new();
    let first = clock.tick(10);
    let stalled = clock.tick(10);
    let regressed = clock.tick(9);
    let advanced = clock.tick(11);
    assert!(first < stalled);
    assert!(stalled < regressed);
    assert!(regressed < advanced);
    assert_eq!(advanced.counter, 0);
}

struct Deny;
struct Permit;

impl Authorizer for Deny {
    fn authorize(&self, _state: &GroupState, _op: &Operation) -> Result<(), AuthzDenial> {
        Err(AuthzDenial {
            code: "denied".to_string(),
            message: "denied".to_string(),
        })
    }
}

impl Authorizer for Permit {
    fn authorize(&self, _state: &GroupState, _op: &Operation) -> Result<(), AuthzDenial> {
        Ok(())
    }
}

#[test]
fn authorization_denies_or_applies_without_cn_perm() {
    let mut state = GroupState::default();
    let mut report = StoreReport::default();
    let denied = submit(&mut state, &Deny, vec![group_create(1)], &mut report);
    assert!(matches!(denied[0], SubmitOutcome::Denied { .. }));
    assert!(state.group.is_none());
    let applied = submit(&mut state, &Permit, vec![group_create(1)], &mut report);
    assert!(matches!(applied[0], SubmitOutcome::Applied(_)));
    assert!(state.group.is_some());
}

#[test]
fn submit_reports_missing_target_quarantine() {
    let (mut state, _) = fold(vec![group_create(0)]);
    let mut report = StoreReport::default();
    let missing = named_entity_with_sort(99, "Missing", 2);
    let outcomes = submit(&mut state, &Permit, vec![missing.clone()], &mut report);

    assert_eq!(outcomes, vec![SubmitOutcome::Quarantined(missing.op_id)]);
    assert_eq!(report.quarantined.len(), 1);
    assert_eq!(report.quarantined[0].op_id, missing.op_id);
    assert_eq!(
        report.quarantined[0].reason,
        QuarantineReason::MissingTarget
    );
    assert_eq!(
        report.quarantined[0].subject,
        Some(SubjectRef::Entity(entity_id(99)))
    );
}

#[test]
fn validation_quarantine_and_wrong_group() {
    let mut bad = entity(1);
    bad.attributes
        .insert(AttrId::new("unknown").expect("attr id"), attr_text("x"));
    let mut wrong_group_op = op(3, OpKind::EntityCreate { entity: entity(2) });
    wrong_group_op.group_id = id(999);
    let mut weighted = edge(1, 1, 2);
    weighted.weight = Some(1.0);
    let ops = vec![
        group_create(0),
        op(1, OpKind::EntityCreate { entity: bad }),
        op(2, OpKind::EntityCreate { entity: entity(1) }),
        wrong_group_op,
        op(4, OpKind::EntityCreate { entity: entity(2) }),
        op(5, OpKind::EdgeCreate { edge: weighted }),
    ];
    let (_state, report) = fold(ops);
    assert!(
        report
            .quarantined
            .iter()
            .any(|q| q.reason == QuarantineReason::FailedValidation && !q.findings.is_empty())
    );
    assert!(
        report
            .quarantined
            .iter()
            .any(|q| q.reason == QuarantineReason::WrongGroup)
    );
}

#[test]
fn trust_revoke_and_custody_rules() {
    let grant = TrustGrant::new(
        id(600),
        person(1),
        person(2),
        TrustScope::All,
        ts(10),
        envelope(),
    );
    let custody = CustodyEvent {
        id: id(700),
        action: CustodyAction::Corrected,
        at: ts(12),
        actor: ActorRef::Human(person(1)),
        note: Some("synthetic".to_string()),
    };
    let ops = vec![
        group_create(0),
        op(1, OpKind::TrustGrantCreate { grant }),
        op(
            2,
            OpKind::TrustGrantRevoke {
                grant: id(600),
                at: ts(9),
            },
        ),
        op(
            3,
            OpKind::TrustGrantRevoke {
                grant: id(600),
                at: ts(10),
            },
        ),
        op(
            4,
            OpKind::CustodyAppend {
                target: CustodyTarget::Group,
                event: custody.clone(),
            },
        ),
        op(
            5,
            OpKind::CustodyAppend {
                target: CustodyTarget::Group,
                event: custody,
            },
        ),
    ];
    let (state, report) = fold(ops);
    assert!(
        report
            .quarantined
            .iter()
            .any(|q| q.reason == QuarantineReason::RevokedBeforeGranted)
    );
    assert_eq!(state.trust_grants[&id(600)].revoked_at, Some(ts(10)));
    assert_eq!(
        state
            .group
            .as_ref()
            .expect("group")
            .provenance
            .custody()
            .len(),
        1
    );
}

fn temp_path(name: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!("cn-store-{name}-{}.jsonl", std::process::id()));
    let _ = std::fs::remove_file(&path);
    path
}

#[test]
fn log_append_reopen_torn_final_and_corrupt_middle() {
    let path = temp_path("log");
    let ops = vec![
        group_create(1),
        op(2, OpKind::EntityCreate { entity: entity(1) }),
    ];
    {
        let (mut log, existing, report) = OpLog::open(&path).expect("open");
        assert!(existing.is_empty());
        assert!(report.warnings.is_empty());
        log.append_batch(&ops).expect("append");
    }
    {
        let (_log, replayed, _) = OpLog::open(&path).expect("reopen");
        assert_eq!(replayed, ops);
    }
    let mut torn_bytes = std::fs::read(&path).expect("read log");
    torn_bytes.extend_from_slice(b"garbage");
    std::fs::write(&path, torn_bytes).expect("write torn log");
    let (_log, recovered, report) = OpLog::open(&path).expect("recover");
    assert_eq!(recovered, ops);
    assert!(
        report
            .warnings
            .iter()
            .any(|w| w.code == "TornLineRecovered")
    );

    let corrupt = temp_path("corrupt");
    let mut corrupt_bytes = serde_json::to_vec(&group_create(1)).expect("serialize op");
    corrupt_bytes.extend_from_slice(b"\nnot-json\n");
    corrupt_bytes.extend_from_slice(&serde_json::to_vec(&group_create(2)).expect("serialize op"));
    corrupt_bytes.extend_from_slice(b"\n");
    std::fs::write(&corrupt, corrupt_bytes).expect("write corrupt");
    assert!(matches!(
        OpLog::open(&corrupt),
        Err(StoreError::CorruptLog { line: 2 })
    ));
}

#[test]
fn op_log_rejects_unsupported_op_schema_major() {
    let path = temp_path("unsupported-schema");
    let mut unsupported = group_create(1);
    unsupported.schema_version = Version::new(9, 0, 0);
    let mut bytes = serde_json::to_vec(&unsupported).expect("serialize op");
    bytes.extend_from_slice(b"\n");
    std::fs::write(&path, bytes).expect("write log");

    assert!(matches!(
        OpLog::open(&path),
        Err(StoreError::UnsupportedSchemaVersion)
    ));
}

#[test]
fn snapshot_round_trip_and_corruption_discard() {
    let path = temp_path("snapshot");
    let (state, _) = fold(vec![
        group_create(1),
        op(2, OpKind::EntityCreate { entity: entity(1) }),
    ]);
    Snapshot::save(&path, &state).expect("save");
    let mut load_report = StoreReport::default();
    let loaded = Snapshot::load(&path, &mut load_report)
        .expect("load")
        .expect("some");
    assert_eq!(public_state(&loaded.0), public_state(&state));
    let mut tip_report = StoreReport::default();
    assert!(
        Snapshot::load_with_log_tip(&path, Some(&group_create(1).sort_key), &mut tip_report)
            .expect("load with shorter log")
            .is_none()
    );
    assert!(
        tip_report
            .warnings
            .iter()
            .any(|finding| finding.code == "SnapshotDiscarded")
    );
    let mut bytes = std::fs::read(&path).expect("read snapshot");
    let last = bytes.len() - 2;
    bytes[last] ^= 1;
    std::fs::write(&path, bytes).expect("write flipped");
    let mut corrupt_report = StoreReport::default();
    assert!(
        Snapshot::load(&path, &mut corrupt_report)
            .expect("load corrupt")
            .is_none()
    );
    assert!(
        corrupt_report
            .warnings
            .iter()
            .any(|finding| finding.code == "SnapshotDiscarded")
    );
}

#[test]
fn snapshot_round_trips_field_clocks_and_seen() {
    let path = temp_path("snapshot-clocks");
    let duplicate = named_entity_with_sort(1, "high", 5);
    let snapshot_ops = vec![
        group_create(1),
        op(2, OpKind::EntityCreate { entity: entity(1) }),
        duplicate.clone(),
    ];
    let (snapshot_state, _) = fold(snapshot_ops.clone());
    Snapshot::save(&path, &snapshot_state).expect("save");
    let lower = named_entity_with_sort(1, "low", 4);

    let mut report = StoreReport::default();
    let (mut loaded, _) = Snapshot::load(&path, &mut report)
        .expect("load")
        .expect("some");
    loaded.apply(lower.clone(), &mut report);
    loaded.apply(duplicate.clone(), &mut report);

    let mut never_snapshotted = snapshot_ops;
    never_snapshotted.push(lower);
    never_snapshotted.push(duplicate);
    let (expected, _) = fold(never_snapshotted);

    assert_eq!(public_state(&loaded), public_state(&expected));
    assert_eq!(
        loaded.entities[&entity_id(1)].attributes[&AttrId::new("name").expect("attr")].value,
        AttributeValue::Text("high".to_string())
    );
    assert!(report.deduped > 0);
}

#[test]
fn hlc_clock_rolls_counter_over_by_advancing_wall_time() {
    let mut clock = HlcClock::from_last_for_test(Hlc {
        wall_ms: 10,
        counter: u32::MAX,
    });
    let rolled = clock.tick(10);
    let next = clock.tick(9);

    assert_eq!(
        rolled,
        Hlc {
            wall_ms: 11,
            counter: 0
        }
    );
    assert!(rolled < next);
}
