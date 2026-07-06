#![allow(clippy::too_many_lines)]

//! I5 note: the ignored measurement gate keeps deterministic generation and
//! instrumentation together so the printed numbers are reproducible.

use std::str::FromStr;
use std::time::Instant;

use cn_model::*;
use cn_store::*;

const RESEARCH: &str =
    include_str!("../../../../fixtures/templates/research-network.template.json");

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

fn person(n: u128) -> PersonId {
    id(20 + n)
}

fn op_id(n: u128) -> OpId {
    id(30 + n)
}

fn entity_id(n: u128) -> EntityId {
    id(10_000 + n)
}

fn edge_id(n: u128) -> EdgeId {
    id(20_000 + n)
}

fn envelope(who: PersonId) -> ProvenanceEnvelope {
    ProvenanceEnvelope::new(Origin::Authored, ActorRef::Human(who), who, ts(1))
        .expect("valid provenance")
}

fn kind(n: u64) -> KindId {
    let names = ["person", "organization", "publication", "study_area"];
    KindId::new(names[(n as usize) % names.len()]).expect("kind")
}

fn attr_names(kind: &KindId) -> Vec<&'static str> {
    match kind.as_str() {
        "person" | "organization" | "study_area" => vec!["display_name"],
        "publication" => vec!["title"],
        _ => Vec::new(),
    }
}

fn group_create() -> Operation {
    let human = person(1);
    let group = Group::new(
        group_id(),
        "Synthetic Research Network",
        TemplateId::new("research_network").expect("template"),
        cn_model::model_schema_version(),
        envelope(human),
        SensitivityTier::T1,
    )
    .expect("group");
    op(
        1,
        OpKind::GroupCreate {
            group,
            template_json: RESEARCH.to_string(),
        },
    )
}

fn op(sort: u128, kind: OpKind) -> Operation {
    let human = person(1);
    let actor = ActorRef::Human(human);
    let op_id = op_id(sort);
    Operation {
        op_id,
        group_id: group_id(),
        actor: actor.clone(),
        responsible_human: human,
        recorded_at: ts(sort as i64),
        sort_key: SortKey::new(
            Hlc {
                wall_ms: sort as i64,
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

fn entity(n: u128) -> Entity {
    let kind = kind(n as u64);
    let mut entity = Entity::new(
        entity_id(n),
        group_id(),
        kind.clone(),
        Circle::Group,
        envelope(person(1)),
        SensitivityTier::T1,
    );
    for attr in attr_names(&kind) {
        entity.attributes.insert(
            AttrId::new(attr).expect("attr"),
            AttributeInstance::new(
                AttributeValue::Text(format!("Synthetic {attr} {n}")),
                Circle::Group,
                envelope(person(1)),
            ),
        );
    }
    entity
}

fn edge(n: u128, from: u128, to: u128) -> Edge {
    Edge::new(
        edge_id(n),
        group_id(),
        KindId::new("coauthored").expect("edge kind"),
        entity_id(from),
        entity_id(to),
        false,
        Circle::Group,
        envelope(person(1)),
        SensitivityTier::T1,
    )
}

#[test]
#[ignore]
fn measurement_gate() {
    let mut ops = Vec::with_capacity(15_002);
    ops.push(group_create());
    // The measured viewer must be an active member or the projection under
    // measurement is empty (Group-visibility values require Group access).
    ops.push(op(
        5_999,
        OpKind::MembershipAdd {
            membership: Membership::new(
                id(40),
                group_id(),
                person(1),
                GroupRole::Member,
                envelope(person(1)),
            ),
        },
    ));
    for n in 1..=5_000 {
        ops.push(op(n + 1, OpKind::EntityCreate { entity: entity(n) }));
    }
    for n in 1..=10_000 {
        let from = ((n % 1_250) + 1) * 4;
        let to = (((n * 37) % 1_250) + 1) * 4;
        ops.push(op(
            n + 6_000,
            OpKind::EdgeCreate {
                edge: edge(n, from, to),
            },
        ));
    }
    let ops_jsonl = ops
        .iter()
        .map(|op| serde_json::to_string(op).expect("serialize op"))
        .collect::<Vec<_>>()
        .join("\n");
    let fold_started = Instant::now();
    let (state, _report) = fold(ops);
    let fold_ms = fold_started.elapsed().as_millis();
    let projection_started = Instant::now();
    let projection = cn_perm::project(
        &state,
        &cn_perm::ViewerContext::Person { person: person(1) },
        0,
    );
    let projection_json = serde_json::to_vec(&projection).expect("serialize projection");
    let projection_ms = projection_started.elapsed().as_millis();
    // Size proxy over the public collections: GroupState itself carries a
    // private tuple-keyed clock map that serde_json cannot key.
    let state_bytes = serde_json::to_vec(&state.entities)
        .expect("serialize entities")
        .len()
        + serde_json::to_vec(&state.edges)
            .expect("serialize edges")
            .len()
        + serde_json::to_vec(&state.memberships)
            .expect("serialize memberships")
            .len()
        + serde_json::to_vec(&state.trust_grants)
            .expect("serialize trust grants")
            .len()
        + serde_json::to_vec(&state.stories)
            .expect("serialize stories")
            .len();
    println!("fold_wall_ms={fold_ms}");
    println!("projection_wall_ms={projection_ms}");
    println!("projection_json_bytes={}", projection_json.len());
    println!("ops_jsonl_bytes={}", ops_jsonl.len());
    println!("state_json_bytes_proxy={state_bytes}");
}
