//! Facilitator role acceptance through the cn-api facade (P3.1/P3.3;
//! facilitator blueprint sections 3-4): the projection cache never serves a
//! facilitator a member's cached projection or vice versa, "facilitator"
//! round-trips on the wire, and the StoryUpdate/CustodyAppend authority
//! changes hold through submit_ops.

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

fn entity_id(n: u128) -> EntityId {
    id(100 + n)
}

fn story_id(n: u128) -> StoryId {
    id(250 + n)
}

fn op_id(n: u128) -> OpId {
    id(300 + n)
}

fn person(n: u128) -> PersonId {
    id(400 + n)
}

fn membership_id(n: u128) -> MembershipId {
    id(500 + n)
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
                { "id": "display_name", "type": "text", "required": true }
            ]
        }],
        "edge_kinds": [{
            "id": "knows",
            "label": "Knows",
            "from": ["person"],
            "to": ["person"],
            "directed": false,
            "weighted": "optional"
        }]
    }"##
    .to_string()
}

fn viewer_person(n: u128) -> String {
    json!({ "kind": "person", "person": person(n) }).to_string()
}

fn op(sort: i64, submitter: PersonId, kind: OpKind) -> Operation {
    let actor = ActorRef::Human(submitter);
    let op_id = op_id(sort as u128);
    Operation {
        op_id,
        group_id: group_id(),
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

fn group() -> Group {
    Group::new(
        group_id(),
        "Research Network",
        TemplateId::new("research").expect("template id"),
        cn_model::model_schema_version(),
        envelope(person(1)),
        SensitivityTier::T1,
    )
    .expect("valid group")
}

fn membership_add(membership: u128, who: u128, role: GroupRole) -> OpKind {
    OpKind::MembershipAdd {
        membership: Membership::new(
            membership_id(membership),
            group_id(),
            person(who),
            role,
            envelope(person(who)),
        ),
    }
}

fn entity_create(n: u128, by: PersonId) -> OpKind {
    let mut entity = Entity::new(
        entity_id(n),
        group_id(),
        KindId::new("person").expect("kind id"),
        Circle::Group,
        envelope(by),
        SensitivityTier::T1,
    );
    entity.attributes.insert(
        AttrId::new("display_name").expect("attr id"),
        AttributeInstance::new(
            AttributeValue::Text(format!("Synthetic Person {n}")),
            Circle::Group,
            envelope(by),
        ),
    );
    OpKind::EntityCreate { entity }
}

fn story_create(n: u128, by: PersonId) -> OpKind {
    OpKind::StoryCreate {
        story: Story::new(
            story_id(n),
            group_id(),
            format!("Synthetic Story {n}"),
            vec![StoryStep {
                entity: entity_id(1),
                narration: "Synthetic step".to_string(),
            }],
            Circle::Group,
            envelope(by),
            SensitivityTier::T1,
        ),
    }
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

/// Loads a group where person(2) is a member, person(3) a facilitator, and
/// person(4) holds both roles; person(1) is governance. Entity 1 and story 1
/// were created by person(1); entity 2 was created by the facilitator.
fn load_api() -> Api {
    let ops = vec![
        op(
            0,
            person(1),
            OpKind::GroupCreate {
                group: group(),
                template_json: template_json(),
            },
        ),
        op(1, person(1), membership_add(1, 1, GroupRole::Governance)),
        op(2, person(1), membership_add(2, 2, GroupRole::Member)),
        op(3, person(1), membership_add(3, 3, GroupRole::Facilitator)),
        op(4, person(1), membership_add(4, 4, GroupRole::Facilitator)),
        op(5, person(1), membership_add(5, 4, GroupRole::Member)),
        op(6, person(1), entity_create(1, person(1))),
        op(7, person(3), entity_create(2, person(3))),
        op(8, person(1), story_create(1, person(1))),
    ];
    let mut api = Api::new();
    ok(&api.load_group_begin(
        &group_id().to_string(),
        &json!({ "kind": "anonymous" }).to_string(),
        &template_json(),
    ));
    ok(&api.load_ops_chunk(&group_id().to_string(), &jsonl(&ops)));
    ok(&api.load_group_commit(&group_id().to_string(), 1_000));
    api
}

fn submit(api: &mut Api, viewer: &str, ops: Vec<Operation>) -> Value {
    ok(&api.submit_ops(
        &group_id().to_string(),
        viewer,
        &serde_json::to_string(&ops).expect("ops json"),
        2_000,
    ))
}

fn entity_ids(projection: &Value) -> Vec<Value> {
    projection["entities"]
        .as_array()
        .expect("entities")
        .iter()
        .map(|entity| entity["id"].clone())
        .collect()
}

#[test]
fn member_and_facilitator_share_read_reach_but_never_share_cache_entries() {
    let mut api = load_api();
    let member = ok(&api.projection(&group_id().to_string(), &viewer_person(2)));
    let facilitator = ok(&api.projection(&group_id().to_string(), &viewer_person(3)));
    let dual = ok(&api.projection(&group_id().to_string(), &viewer_person(4)));

    // No new read reach: a facilitator reads at member level.
    assert_eq!(entity_ids(&member), entity_ids(&facilitator));
    assert_eq!(member["stories"], facilitator["stories"]);
    assert_eq!(entity_ids(&member).len(), 2);

    // Distinct cache keys for member, facilitator, and dual-role viewers.
    let member_fp = member["viewer_fingerprint"].as_str().expect("fingerprint");
    let facilitator_fp = facilitator["viewer_fingerprint"]
        .as_str()
        .expect("fingerprint");
    let dual_fp = dual["viewer_fingerprint"].as_str().expect("fingerprint");
    assert_ne!(member_fp, facilitator_fp);
    assert_ne!(member_fp, dual_fp);
    assert_ne!(facilitator_fp, dual_fp);
}

#[test]
fn story_update_submit_allowed_for_facilitator_denied_for_member() {
    let mut api = load_api();
    let update = |sort: i64, submitter: PersonId| {
        op(
            sort,
            submitter,
            OpKind::StoryUpdate {
                story: story_id(1),
                title: Some("Synthetic updated title".to_string()),
                steps: None,
            },
        )
    };

    let denied = submit(&mut api, &viewer_person(2), vec![update(20, person(2))]);
    assert_eq!(denied["revision"], 0);
    assert_eq!(
        denied["outcomes"][0]["Denied"]["denial"]["code"],
        "not_facilitator_or_governance"
    );

    let applied = submit(&mut api, &viewer_person(3), vec![update(21, person(3))]);
    assert_eq!(applied["revision"], 1);
    assert_eq!(applied["outcomes"][0]["Applied"], op_id(21).to_string());
    let projection = ok(&api.projection(&group_id().to_string(), &viewer_person(2)));
    assert_eq!(projection["stories"][0]["title"], "Synthetic updated title");
}

#[test]
fn custody_submit_respects_facilitator_created_records_rule() {
    let mut api = load_api();
    let append = |sort: i64, submitter: PersonId, target: CustodyTarget| {
        op(
            sort,
            submitter,
            OpKind::CustodyAppend {
                target,
                event: CustodyEvent {
                    id: id(900 + sort as u128),
                    action: CustodyAction::Corrected,
                    at: ts(5),
                    actor: ActorRef::Human(submitter),
                    note: Some("Synthetic custody".to_string()),
                },
            },
        )
    };

    // Facilitator-only viewer: denied on a governance-created record.
    let denied = submit(
        &mut api,
        &viewer_person(3),
        vec![append(30, person(3), CustodyTarget::Entity(entity_id(1)))],
    );
    assert_eq!(
        denied["outcomes"][0]["Denied"]["denial"]["code"],
        "custody_not_own_record"
    );

    // Facilitator-only viewer: allowed on their own created record.
    let applied = submit(
        &mut api,
        &viewer_person(3),
        vec![append(31, person(3), CustodyTarget::Entity(entity_id(2)))],
    );
    assert_eq!(applied["outcomes"][0]["Applied"], op_id(31).to_string());

    // Dual-role viewer keeps member reach over the same governance-created
    // record.
    let dual = submit(
        &mut api,
        &viewer_person(4),
        vec![append(32, person(4), CustodyTarget::Entity(entity_id(1)))],
    );
    assert_eq!(dual["outcomes"][0]["Applied"], op_id(32).to_string());
}
