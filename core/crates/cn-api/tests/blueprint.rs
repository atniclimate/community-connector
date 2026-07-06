#![allow(clippy::too_many_lines)]

//! I5 note: this acceptance module intentionally keeps the cn-api blueprint
//! obligations together because the synthetic operation builder is shared.

use std::str::FromStr;

use cn_api::Api;
use cn_model::*;
use cn_store::*;
use serde_json::{Value, json};

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

fn ts(ms: i64) -> Timestamp {
    Timestamp(ms)
}

fn group_id() -> GroupId {
    id(10)
}

fn other_group_id() -> GroupId {
    id(11)
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

fn person(n: u128) -> PersonId {
    id(400 + n)
}

fn attr_id(value: &str) -> AttrId {
    AttrId::new(value).expect("valid attr")
}

fn kind_id(value: &str) -> KindId {
    KindId::new(value).expect("valid kind")
}

fn envelope(who: PersonId) -> ProvenanceEnvelope {
    ProvenanceEnvelope::new(Origin::Authored, ActorRef::Human(who), who, ts(1))
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
                { "id": "score", "type": "number" },
                { "id": "secret", "type": "text" }
            ]
        }],
        "edge_kinds": [{
            "id": "knows",
            "label": "Knows",
            "from": ["person"],
            "to": ["person"],
            "directed": false,
            "weighted": "optional"
        }],
        "theme": { "mode": "light", "roles": { "kind-1": "#224466" } }
    }"##
    .to_string()
}

fn viewer_anon() -> String {
    json!({ "kind": "anonymous" }).to_string()
}

fn viewer_person(n: u128) -> String {
    json!({ "kind": "person", "person": person(n) }).to_string()
}

fn actor(who: PersonId) -> ActorRef {
    ActorRef::Human(who)
}

fn op(group_id: GroupId, sort: i64, submitter: PersonId, kind: OpKind) -> Operation {
    let actor = actor(submitter);
    let op_id = op_id(sort as u128);
    Operation {
        op_id,
        group_id,
        actor: actor.clone(),
        responsible_human: submitter,
        recorded_at: ts(sort),
        sort_key: SortKey::new(
            Hlc {
                wall_ms: sort,
                counter: 0,
            },
            &actor,
            op_id,
        ),
        template_version: cn_model::model_schema_version(),
        kind,
        schema_version: cn_model::model_schema_version(),
    }
}

fn entity(n: u128, owner: Option<PersonId>, visibility: Circle) -> Entity {
    let who = owner.unwrap_or_else(|| person(1));
    let mut entity = Entity::new(
        entity_id(n),
        group_id(),
        kind_id("person"),
        visibility,
        envelope(who),
        SensitivityTier::T0,
    );
    entity.owner = owner;
    entity.attributes.insert(
        attr_id("display_name"),
        AttributeInstance::new(
            AttributeValue::Text(format!("Synthetic Person {n}")),
            visibility,
            envelope(who),
        ),
    );
    entity
}

fn t3_attr_entity(n: u128, owner: PersonId) -> Entity {
    let mut entity = entity(n, Some(owner), Circle::Public);
    let mut secret = AttributeInstance::new(
        AttributeValue::Text("Synthetic secret".to_string()),
        Circle::Public,
        envelope(owner),
    );
    secret
        .tighten_tier(SensitivityTier::T0, SensitivityTier::T3)
        .expect("tier tightens");
    entity.attributes.insert(attr_id("secret"), secret);
    entity
}

fn edge(n: u128, from: u128, to: u128) -> Edge {
    let mut edge = Edge::new(
        edge_id(n),
        group_id(),
        kind_id("knows"),
        entity_id(from),
        entity_id(to),
        false,
        Circle::Public,
        envelope(person(1)),
        SensitivityTier::T0,
    );
    edge.set_weight(Some(2.0)).expect("finite weight");
    edge
}

fn jsonl(ops: &[Operation]) -> String {
    ops.iter()
        .map(|op| serde_json::to_string(op).expect("serialize op"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn ok(json: &str) -> Value {
    let value: Value = serde_json::from_str(json).expect("valid envelope json");
    assert!(value.get("ok").is_some() ^ value.get("err").is_some());
    value
        .get("ok")
        .cloned()
        .unwrap_or_else(|| panic!("ok envelope: {json}"))
}

fn err(json: &str) -> Value {
    let value: Value = serde_json::from_str(json).expect("valid envelope json");
    assert!(value.get("ok").is_some() ^ value.get("err").is_some());
    value.get("err").cloned().expect("err envelope")
}

fn code(json: &str) -> String {
    err(json)["code"].as_str().expect("code").to_string()
}

fn constraints(respect_direction: bool) -> Value {
    json!({
        "allowed_edge_kinds": null,
        "max_hops": null,
        "weighted": false,
        "respect_direction": respect_direction
    })
}

fn load_api(ops: &[Operation]) -> Api {
    let mut api = Api::new();
    ok(&api.load_group_begin(&group_id().to_string(), &viewer_anon(), &template_json()));
    if !ops.is_empty() {
        ok(&api.load_ops_chunk(&group_id().to_string(), &jsonl(ops)));
    }
    ok(&api.load_group_commit(&group_id().to_string(), 1_000));
    api
}

#[test]
fn envelope_discipline_and_basic_errors() {
    let mut api = Api::new();
    ok(&api.core_info());
    assert_eq!(
        code(&api.projection(&group_id().to_string(), &viewer_anon())),
        "not_found"
    );
    assert_eq!(
        code(&api.load_group_begin(&group_id().to_string(), "{", &template_json())),
        "invalid_viewer"
    );
    assert_eq!(
        code(&api.load_group_begin(&group_id().to_string(), &viewer_anon(), "{")),
        "invalid_json"
    );
    ok(&api.load_group_begin(&group_id().to_string(), &viewer_anon(), &template_json()));
    assert_eq!(
        code(&api.load_group_begin(&group_id().to_string(), &viewer_anon(), &template_json())),
        "group_exists"
    );
    assert_eq!(
        code(&api.load_ops_chunk(&other_group_id().to_string(), "")),
        "not_found"
    );
}

#[test]
fn wrong_group_submit_is_deterministic_quarantine_without_revision_bump() {
    let mut api = load_api(&[]);
    let mut wrong = op(
        other_group_id(),
        10,
        person(1),
        OpKind::EntityCreate {
            entity: entity(1, Some(person(1)), Circle::Public),
        },
    );
    wrong.kind = OpKind::EntityCreate {
        entity: entity(1, Some(person(1)), Circle::Public),
    };
    let report = ok(&api.submit_ops(
        &group_id().to_string(),
        &viewer_person(1),
        &serde_json::to_string(&vec![wrong]).expect("ops json"),
        2_000,
    ));
    assert_eq!(report["revision"], 0);
    assert_eq!(report["outcomes"][0]["Quarantined"], op_id(10).to_string());
}

#[test]
fn hidden_and_absent_are_identical_not_found() {
    let ops = vec![
        op(
            group_id(),
            2,
            person(1),
            OpKind::EntityCreate {
                entity: entity(1, Some(person(1)), Circle::Private),
            },
        ),
        op(
            group_id(),
            3,
            person(1),
            OpKind::EntityCreate {
                entity: entity(2, None, Circle::Public),
            },
        ),
    ];
    let mut api = load_api(&ops);
    let hidden = api.entity_detail(
        &group_id().to_string(),
        &viewer_anon(),
        &entity_id(1).to_string(),
    );
    let absent = api.entity_detail(
        &group_id().to_string(),
        &viewer_anon(),
        &entity_id(99).to_string(),
    );
    assert_eq!(hidden, absent);

    let hidden_path = api.query_paths(
        &group_id().to_string(),
        &viewer_anon(),
        &json!({ "from": entity_id(2), "to": entity_id(1), "constraints": constraints(false) })
            .to_string(),
    );
    let absent_path = api.query_paths(
        &group_id().to_string(),
        &viewer_anon(),
        &json!({ "from": entity_id(2), "to": entity_id(99), "constraints": constraints(false) })
            .to_string(),
    );
    assert_eq!(hidden_path, absent_path);
}

#[test]
fn streaming_load_two_chunks_matches_one_chunk_and_commit_rules() {
    let ops = vec![
        op(
            group_id(),
            2,
            person(1),
            OpKind::EntityCreate {
                entity: entity(1, None, Circle::Public),
            },
        ),
        op(
            group_id(),
            3,
            person(1),
            OpKind::EntityCreate {
                entity: entity(2, None, Circle::Public),
            },
        ),
        op(
            group_id(),
            4,
            person(1),
            OpKind::EdgeCreate {
                edge: edge(1, 1, 2),
            },
        ),
    ];
    let mut streamed = Api::new();
    ok(&streamed.load_group_begin(&group_id().to_string(), &viewer_anon(), &template_json()));
    ok(&streamed.load_ops_chunk(&group_id().to_string(), &jsonl(&ops[0..1])));
    ok(&streamed.load_ops_chunk(&group_id().to_string(), &jsonl(&ops[1..])));
    ok(&streamed.load_group_commit(&group_id().to_string(), 1_000));

    let mut one_shot = load_api(&ops);
    assert_eq!(
        streamed.projection(&group_id().to_string(), &viewer_anon()),
        one_shot.projection(&group_id().to_string(), &viewer_anon())
    );
    assert_eq!(
        code(&streamed.load_group_commit(&group_id().to_string(), 1_000)),
        "load_not_begun"
    );
    let mut api = Api::new();
    assert_eq!(
        code(&api.load_group_commit(&group_id().to_string(), 1_000)),
        "not_found"
    );
}

#[test]
fn revision_semantics_and_cache_invalidation() {
    let mut api = load_api(&[]);
    let before = ok(&api.projection(&group_id().to_string(), &viewer_person(1)));
    assert_eq!(before["revision"], 0);
    let create = op(
        group_id(),
        2,
        person(1),
        OpKind::EntityCreate {
            entity: entity(1, Some(person(1)), Circle::Public),
        },
    );
    let submitted = ok(&api.submit_ops(
        &group_id().to_string(),
        &viewer_person(1),
        &serde_json::to_string(&vec![create]).expect("ops json"),
        2_000,
    ));
    assert_eq!(submitted["revision"], 1);
    let after = ok(&api.projection(&group_id().to_string(), &viewer_person(1)));
    assert_eq!(after["revision"], 1);
    assert_eq!(after["entities"].as_array().expect("entities").len(), 1);

    let denied = op(
        group_id(),
        3,
        person(2),
        OpKind::EntityCreate {
            entity: entity(2, Some(person(1)), Circle::Public),
        },
    );
    let denied_report = ok(&api.submit_ops(
        &group_id().to_string(),
        &viewer_person(2),
        &serde_json::to_string(&vec![denied]).expect("ops json"),
        3_000,
    ));
    assert_eq!(denied_report["revision"], 1);
}

#[test]
fn entity_detail_settings_only_for_owner_viewer() {
    let ops = vec![op(
        group_id(),
        2,
        person(1),
        OpKind::EntityCreate {
            entity: entity(1, Some(person(1)), Circle::Public),
        },
    )];
    let mut api = load_api(&ops);
    let own = ok(&api.entity_detail(
        &group_id().to_string(),
        &viewer_person(1),
        &entity_id(1).to_string(),
    ));
    assert!(own["attributes"]["display_name"]["visibility"].is_string());
    assert!(own["attributes"]["display_name"]["tier"].is_string());
    let other = ok(&api.entity_detail(
        &group_id().to_string(),
        &viewer_person(2),
        &entity_id(1).to_string(),
    ));
    assert!(other["attributes"]["display_name"]["visibility"].is_null());
    assert!(other["attributes"]["display_name"]["tier"].is_null());
}

#[test]
fn export_snapshot_narrows_kinds_and_removes_t3() {
    let ops = vec![
        op(
            group_id(),
            2,
            person(1),
            OpKind::EntityCreate {
                entity: t3_attr_entity(1, person(1)),
            },
        ),
        op(
            group_id(),
            3,
            person(1),
            OpKind::EntityCreate {
                entity: entity(2, None, Circle::Public),
            },
        ),
        op(
            group_id(),
            4,
            person(1),
            OpKind::EdgeCreate {
                edge: edge(1, 1, 2),
            },
        ),
    ];
    let mut api = load_api(&ops);
    let projection = ok(&api.projection(&group_id().to_string(), &viewer_person(1)));
    assert!(projection.to_string().contains("Synthetic secret"));
    let export = ok(&api.export_snapshot(
        &group_id().to_string(),
        &viewer_person(1),
        &json!({ "kinds": ["person"] }).to_string(),
    ));
    assert!(!export.to_string().contains("Synthetic secret"));
    let ids: Vec<_> = export["projection"]["entities"]
        .as_array()
        .expect("entities")
        .iter()
        .map(|entity| entity["id"].clone())
        .collect();
    for edge in export["projection"]["edges"].as_array().expect("edges") {
        assert!(ids.contains(&edge["from"]));
        assert!(ids.contains(&edge["to"]));
    }
}

#[test]
fn round_trip_fixture_ops_and_graph_calls_succeed() {
    let ops = vec![
        op(
            group_id(),
            2,
            person(1),
            OpKind::EntityCreate {
                entity: entity(1, None, Circle::Public),
            },
        ),
        op(
            group_id(),
            3,
            person(1),
            OpKind::EntityCreate {
                entity: entity(2, None, Circle::Public),
            },
        ),
        op(
            group_id(),
            4,
            person(1),
            OpKind::EdgeCreate {
                edge: edge(1, 1, 2),
            },
        ),
    ];
    let mut api = load_api(&ops);
    ok(&api.projection(&group_id().to_string(), &viewer_anon()));
    ok(&api.search(
        &group_id().to_string(),
        &viewer_anon(),
        &json!({ "text": "Synthetic", "kinds": null, "limit": 10 }).to_string(),
    ));
    ok(&api.query_paths(
        &group_id().to_string(),
        &viewer_anon(),
        &json!({
            "from": entity_id(1),
            "to": entity_id(2),
            "constraints": constraints(false)
        })
        .to_string(),
    ));
    ok(&api.query_neighborhood(
        &group_id().to_string(),
        &viewer_anon(),
        &json!({ "center": entity_id(1), "hops": 1, "constraints": constraints(false) })
            .to_string(),
    ));
    ok(&api.validation_report(&group_id().to_string(), &viewer_anon()));
}
