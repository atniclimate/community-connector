//! Read-only intake facade tests (blueprint sections 3 and 6): typed
//! results per viewer class and the five-class no-leak EXTENSION - near-dup
//! candidates are bounded by the REAL viewer projection computed in-core,
//! so a facilitator can never receive a candidate referencing an entity
//! they cannot already see, while a wider viewer (here: governance holding
//! the owner's trust) does.

use std::str::FromStr;

use serde_json::{Value, json};

use cn_ingest::{QueueRecord, SubmissionSource};
use cn_model::{
    ActorRef, AttrId, AttributeInstance, AttributeValue, Circle, Entity, Group, GroupRole,
    Lifecycle, Membership, Origin, ProvenanceEnvelope, SensitivityTier, TemplateId, Timestamp,
    TrustGrant, TrustScope,
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

fn op(sort: i64, kind: OpKind) -> Operation {
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

fn membership(sort: i64, membership_id: u128, person: u128, role: GroupRole) -> Operation {
    let mut membership = Membership::new(id(membership_id), id(10), id(person), role, envelope(1));
    membership.lifecycle = Lifecycle::Active;
    op(sort, OpKind::MembershipAdd { membership })
}

/// The fixture graph: governance person(1), facilitator person(2), member
/// person(5) owning a TRUSTED-visibility "Jo Doe" entity, and a trust
/// grant person(5) -> person(1). Governance reaches the Trusted circle for
/// that entity; the facilitator reaches only Group and cannot see it.
fn ops_jsonl() -> String {
    let group = Group::new(
        id(10),
        "Synthetic Group",
        TemplateId::new("research").expect("template id"),
        semver::Version::new(0, 1, 0),
        envelope(1),
        SensitivityTier::T1,
    )
    .expect("valid group");
    let mut hidden = Entity::new(
        id(100),
        id(10),
        cn_model::KindId::new("person").expect("kind"),
        Circle::Trusted,
        envelope(5),
        SensitivityTier::T1,
    );
    hidden.owner = Some(id(5));
    hidden.attributes.insert(
        AttrId::new("display_name").expect("attr"),
        AttributeInstance::new(
            AttributeValue::Text("Jo Doe".to_string()),
            Circle::Trusted,
            envelope(5),
        ),
    );
    hidden.attributes.insert(
        AttrId::new("affiliation").expect("attr"),
        AttributeInstance::new(
            AttributeValue::Tags(["River Alliance".to_string()].into_iter().collect()),
            Circle::Trusted,
            envelope(5),
        ),
    );
    let grant = TrustGrant::new(
        id(200),
        id(5),
        id(1),
        TrustScope::Group(id(10)),
        Timestamp(1),
        envelope(5),
    );
    let ops = [
        op(
            1,
            OpKind::GroupCreate {
                group,
                template_json: template_json(),
            },
        ),
        membership(2, 40, 1, GroupRole::Governance),
        membership(3, 41, 2, GroupRole::Facilitator),
        membership(4, 42, 5, GroupRole::Member),
        op(5, OpKind::EntityCreate { entity: hidden }),
        op(6, OpKind::TrustGrantCreate { grant }),
    ];
    ops.iter()
        .map(|op| serde_json::to_string(op).expect("op serializes"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn viewer(person: u128) -> String {
    json!({ "kind": "person", "person": uuid_string(person) }).to_string()
}

fn loaded_api() -> cn_api::Api {
    let mut api = cn_api::Api::new();
    let group = uuid_string(10);
    let begin = api.load_group_begin(&group, &viewer(1), &template_json());
    assert!(!begin.contains("\"err\""), "begin: {begin}");
    let chunk = api.load_ops_chunk(&group, &ops_jsonl());
    assert!(!chunk.contains("\"err\""), "chunk: {chunk}");
    let commit = api.load_group_commit(&group, 5_000);
    assert!(!commit.contains("\"err\""), "commit: {commit}");
    api
}

fn queue_record() -> QueueRecord {
    QueueRecord::new(
        "rec-1".to_string(),
        Timestamp(10),
        SubmissionSource::InApp {},
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
                "display_name": "Jo Doe",
                "affiliation": ["River Alliance"]
            }
        }),
    )
    .expect("record builds")
}

fn near_dup_request() -> String {
    json!({
        "payload": queue_record().payload,
        "queue_payloads": [["rec-q", { "fields": { "display_name": "Jo A. Doe" } }]],
        "name_attrs": ["display_name"],
        "affiliation_attrs": ["affiliation"]
    })
    .to_string()
}

fn parse(envelope: &str) -> Value {
    let value: Value = serde_json::from_str(envelope).expect("envelope is JSON");
    assert!(
        value.get("err").is_none(),
        "expected ok envelope, got: {envelope}"
    );
    value.get("ok").cloned().expect("ok payload present")
}

#[test]
fn near_dup_candidates_are_bounded_by_the_viewers_projection() {
    let mut api = loaded_api();
    let group = uuid_string(10);

    // Governance holds the owner's trust: the hidden graph entity IS a
    // candidate, with transparent reasons.
    let wide = parse(&api.intake_near_duplicates(&group, &viewer(1), &near_dup_request()));
    let wide_hits = wide.as_array().expect("candidate list");
    assert_eq!(wide_hits.len(), 2, "graph entity + queue record: {wide}");
    let reasons = wide.to_string();
    assert!(reasons.contains("exact normalized name match"));
    assert!(reasons.contains("shared affiliation"));
    assert!(reasons.contains(&uuid_string(100)), "graph candidate id");

    // The facilitator cannot see the trusted entity: the SAME query yields
    // NO graph candidate - only the queue-side match, which is their own
    // pending-review data. The projection is the facilitator's own,
    // computed in-core (no-leak extension, blueprint section 6).
    let narrow = parse(&api.intake_near_duplicates(&group, &viewer(2), &near_dup_request()));
    let narrow_hits = narrow.as_array().expect("candidate list");
    assert_eq!(narrow_hits.len(), 1, "queue match only: {narrow}");
    assert!(
        !narrow.to_string().contains(&uuid_string(100)),
        "no hidden-entity reference for the facilitator"
    );
    assert!(narrow.to_string().contains("rec-q"));

    // Anonymous reaches only Public: no graph candidate either.
    let anonymous = parse(&api.intake_near_duplicates(
        &group,
        &json!({ "kind": "anonymous" }).to_string(),
        &near_dup_request(),
    ));
    assert!(
        !anonymous.to_string().contains(&uuid_string(100)),
        "anonymous sees no graph candidate"
    );
}

#[test]
fn validate_record_reports_kind_and_template_fit() {
    let api = loaded_api();
    let group = uuid_string(10);
    let record_json = serde_json::to_string(&queue_record()).expect("record serializes");

    let ok = parse(&api.intake_validate_record(&group, &record_json));
    assert_eq!(ok["kind"], "person");
    assert_eq!(ok["report"]["errors"].as_array().expect("errors").len(), 0);

    // Checksum tampering is a typed error envelope, not a report.
    let mut tampered = queue_record();
    tampered.payload["fields"]["display_name"] = json!("Tampered");
    let tampered_json = serde_json::to_string(&tampered).expect("serializes");
    let err = api.intake_validate_record(&group, &tampered_json);
    assert!(err.contains("\"err\""), "typed error: {err}");
    assert!(err.contains("checksum"), "names the failure: {err}");

    // Unknown MAJOR queue version is rejected loudly with its own code.
    let mut future = serde_json::to_value(queue_record()).expect("value");
    future["queue_record_version"] = json!("9.0.0");
    let future_json = future.to_string();
    let rejected = api.intake_validate_record(&group, &future_json);
    assert!(
        rejected.contains("unsupported_schema_version"),
        "unknown major keeps its identity: {rejected}"
    );
}

#[test]
fn dedup_check_classifies_against_supplied_keys() {
    let api = loaded_api();
    let record_json = serde_json::to_string(&queue_record()).expect("serializes");

    let fresh = parse(&api.intake_dedup_check(&record_json, "[]"));
    assert_eq!(fresh["verdict"], "fresh");

    // Same submission_id + same payload hash: semantic replay.
    let same_key = json!([{
        "submission_id": "sub-1",
        "payload_hash": queue_record().payload_hash
    }])
    .to_string();
    let replay = parse(&api.intake_dedup_check(&record_json, &same_key));
    assert_eq!(replay["verdict"], "semantic_replay");

    // Same submission_id + different payload hash: conflict, never a drop.
    let conflicting = json!([{
        "submission_id": "sub-1",
        "payload_hash": "different"
    }])
    .to_string();
    let conflict = parse(&api.intake_dedup_check(&record_json, &conflicting));
    assert_eq!(conflict["verdict"], "conflict");
}
