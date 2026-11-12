//! Hidden must be indistinguishable from absent (ADR-003 A-B4) for every
//! graph query and measure: folding the same group with extra entities and
//! edges the viewer cannot see must give the viewer identical results.
//! Synthetic data only (I1).

use std::str::FromStr;

use cn_graph::*;
use cn_model::*;
use cn_perm::{Projection, ViewerContext, project};
use cn_store::*;

const RESEARCH: &str =
    include_str!("../../../../fixtures/templates/research-network.template.json");

fn id<T>(n: u128) -> T
where
    T: FromStr,
    T::Err: std::fmt::Debug,
{
    let hex = format!("{n:032x}");
    let text = format!(
        "{}-{}-{}-{}-{}",
        &hex[0..8],
        &hex[8..12],
        &hex[12..16],
        &hex[16..20],
        &hex[20..32]
    );
    T::from_str(&text).expect("valid deterministic id")
}

fn group_id() -> GroupId {
    id(10)
}

fn person(n: u128) -> PersonId {
    id(20 + n)
}

fn entity_id(n: u128) -> EntityId {
    id(10_000 + n)
}

fn kind(value: &str) -> KindId {
    KindId::new(value).expect("valid kind")
}

fn envelope() -> ProvenanceEnvelope {
    ProvenanceEnvelope::new(
        Origin::Authored,
        ActorRef::Human(person(1)),
        person(1),
        Timestamp(1),
    )
    .expect("valid provenance")
}

fn op(sort: u128, kind: OpKind) -> Operation {
    let actor = ActorRef::Human(person(1));
    let op_id: OpId = id(1_000 + sort);
    Operation {
        op_id,
        group_id: group_id(),
        actor: actor.clone(),
        responsible_human: person(1),
        recorded_at: Timestamp(sort as i64),
        sort_key: SortKey::new(
            Hlc {
                wall_ms: sort as i64,
                counter: 0,
            },
            &actor,
            op_id,
        ),
        template_version: model_schema_version(),
        kind,
        schema_version: model_schema_version(),
    }
}

fn group_ops() -> Vec<Operation> {
    let group = Group::new(
        group_id(),
        "Synthetic Research Network",
        TemplateId::new("research_network").expect("template"),
        model_schema_version(),
        envelope(),
        SensitivityTier::T1,
    )
    .expect("group");
    let mut ops = vec![op(
        1,
        OpKind::GroupCreate {
            group,
            template_json: RESEARCH.to_string(),
        },
    )];
    for n in 1..=2 {
        let membership = Membership::new(
            id(40 + n),
            group_id(),
            person(n),
            GroupRole::Member,
            envelope(),
        );
        ops.push(op(1 + n, OpKind::MembershipAdd { membership }));
    }
    ops
}

/// An entity with a searchable display name; `owner` and `presence` decide
/// who may see it.
fn entity(n: u128, kind_id: &str, presence: Circle, owner: Option<PersonId>) -> Operation {
    let mut entity = Entity::new(
        entity_id(n),
        group_id(),
        kind(kind_id),
        presence,
        envelope(),
        SensitivityTier::T1,
    );
    entity.owner = owner;
    entity.attributes.insert(
        AttrId::new("display_name").expect("attr"),
        AttributeInstance::new(
            AttributeValue::Text(format!("Synthetic {kind_id} {n}")),
            Circle::Group,
            envelope(),
        ),
    );
    op(100 + n, OpKind::EntityCreate { entity })
}

fn edge(n: u128, kind_id: &str, from: u128, to: u128, visibility: Circle) -> Operation {
    let edge = Edge::new(
        id(20_000 + n),
        group_id(),
        kind(kind_id),
        entity_id(from),
        entity_id(to),
        kind_id != "coauthored",
        visibility,
        envelope(),
        SensitivityTier::T1,
    );
    op(200 + n, OpKind::EdgeCreate { edge })
}

/// Visible to person(1): people 1-5 and institutions 6-7, in two
/// components, {1,2,3,6,7} and {4,5}.
fn visible_ops() -> Vec<Operation> {
    let mut ops = group_ops();
    for n in 1..=5 {
        ops.push(entity(n, "person", Circle::Group, None));
    }
    for n in 6..=7 {
        ops.push(entity(n, "organization", Circle::Group, None));
    }
    ops.extend([
        edge(1, "coauthored", 1, 2, Circle::Group),
        edge(2, "coauthored", 2, 3, Circle::Group),
        edge(3, "coauthored", 4, 5, Circle::Group),
        edge(4, "affiliated_with", 1, 6, Circle::Group),
        edge(5, "affiliated_with", 3, 6, Circle::Group),
        edge(6, "affiliated_with", 3, 7, Circle::Group),
    ]);
    ops
}

/// Adds what person(1) must not see: a Private-presence person, a
/// Trusted-presence person and a Private institution (all owned by
/// person(2), who granted person(1) no trust), ties to them that would
/// shorten paths and change shared affiliations, and a Private tie between
/// two visible people.
fn with_hidden_ops() -> Vec<Operation> {
    let mut ops = visible_ops();
    ops.extend([
        entity(8, "person", Circle::Private, Some(person(2))),
        entity(9, "person", Circle::Trusted, Some(person(2))),
        entity(10, "organization", Circle::Private, Some(person(2))),
        edge(11, "coauthored", 3, 8, Circle::Group),
        edge(12, "coauthored", 8, 4, Circle::Group),
        edge(13, "coauthored", 1, 9, Circle::Group),
        edge(14, "coauthored", 9, 5, Circle::Group),
        edge(15, "affiliated_with", 1, 10, Circle::Group),
        edge(16, "affiliated_with", 2, 10, Circle::Group),
        edge(17, "coauthored", 2, 5, Circle::Private),
    ]);
    ops
}

fn projection_for(ops: Vec<Operation>, viewer: PersonId) -> Projection {
    let (state, report) = fold(ops);
    assert!(
        report.quarantined.is_empty() && report.warnings.is_empty(),
        "synthetic fold must be clean: {report:?}"
    );
    project(&state, &ViewerContext::Person { person: viewer }, 1)
}

fn constraint_sets() -> Vec<PathConstraints> {
    let base = PathConstraints::default();
    vec![
        base.clone(),
        PathConstraints {
            weighted: true,
            ..base.clone()
        },
        PathConstraints {
            respect_direction: true,
            ..base.clone()
        },
        PathConstraints {
            allowed_edge_kinds: Some(vec![kind("coauthored")]),
            max_hops: Some(2),
            ..base
        },
    ]
}

fn assert_measures_match(plain: &GraphIndex, hidden: &GraphIndex, ids: &[EntityId]) {
    assert_eq!(degree_measures(plain), degree_measures(hidden));
    assert_eq!(betweenness(plain), betweenness(hidden));
    assert_eq!(eccentricity(plain), eccentricity(hidden));
    assert_eq!(continuance(plain), continuance(hidden));
    let membership = kind("affiliated_with");
    for &a in ids {
        for &b in ids {
            assert_eq!(
                shared_committee_jaccard(plain, a, b, &membership),
                shared_committee_jaccard(hidden, a, b, &membership),
            );
        }
    }
}

fn assert_queries_match(plain: &GraphIndex, hidden: &GraphIndex, ids: &[EntityId]) {
    for c in constraint_sets() {
        for &a in ids {
            assert_eq!(
                neighborhood(plain, a, 4, &c),
                neighborhood(hidden, a, 4, &c)
            );
            for &b in ids {
                assert_eq!(
                    shortest_path(plain, a, b, &c),
                    shortest_path(hidden, a, b, &c)
                );
            }
        }
    }
}

fn assert_search_matches(plain: &Projection, hidden: &Projection) {
    for text in ["synthetic", "person", "organization 1", "8", "9"] {
        let q = SearchQuery {
            text: text.to_string(),
            kinds: None,
            limit: 50,
        };
        assert_eq!(search(plain, &q), search(hidden, &q), "search {text:?}");
    }
}

#[test]
fn queries_and_measures_ignore_entities_and_edges_hidden_from_the_viewer() {
    let plain = projection_for(visible_ops(), person(1));
    let hidden = projection_for(with_hidden_ops(), person(1));
    let plain_idx = GraphIndex::build(&plain);
    let hidden_idx = GraphIndex::build(&hidden);
    let ids: Vec<EntityId> = plain.entities.iter().map(|e| e.id).collect();
    assert_eq!(ids.len(), 7, "all visible entities projected");

    assert_measures_match(&plain_idx, &hidden_idx, &ids);
    assert_queries_match(&plain_idx, &hidden_idx, &ids);
    assert_search_matches(&plain, &hidden);
    for &center in &ids {
        let req = IntroductionRequest::new(center);
        assert_eq!(
            suggest_introductions(&plain, &plain_idx, &req),
            suggest_introductions(&hidden, &hidden_idx, &req)
        );
    }

    // A hidden id answers exactly like an id that never existed.
    let never: EntityId = entity_id(99);
    for n in 8..=10 {
        let hidden_id = entity_id(n);
        assert_eq!(
            suggest_introductions(&hidden, &hidden_idx, &IntroductionRequest::new(hidden_id)),
            suggest_introductions(&hidden, &hidden_idx, &IntroductionRequest::new(never))
        );
        let c = PathConstraints::default();
        assert_eq!(
            shortest_path(&hidden_idx, ids[0], hidden_id, &c),
            shortest_path(&hidden_idx, ids[0], never, &c)
        );
        assert_eq!(
            neighborhood(&hidden_idx, hidden_id, 2, &c),
            neighborhood(&hidden_idx, never, 2, &c)
        );
    }
}

/// Guards against a vacuous fixture: the owner of the hidden entities sees
/// them, and the extra ties then change paths, measures and affiliations.
#[test]
fn hidden_fixture_would_change_results_if_it_were_visible() {
    let plain = GraphIndex::build(&projection_for(visible_ops(), person(2)));
    let owner = GraphIndex::build(&projection_for(with_hidden_ops(), person(2)));
    let c = PathConstraints::default();
    let (p3, p4) = (entity_id(3), entity_id(4));
    assert_eq!(shortest_path(&plain, p3, p4, &c), Ok(None));
    assert!(shortest_path(&owner, p3, p4, &c).expect("known").is_some());
    assert_ne!(betweenness(&plain), betweenness(&owner));
    assert_ne!(continuance(&plain), continuance(&owner));
    let membership = kind("affiliated_with");
    assert_ne!(
        shared_committee_jaccard(&plain, entity_id(1), entity_id(2), &membership),
        shared_committee_jaccard(&owner, entity_id(1), entity_id(2), &membership),
    );
    let (plain_p, owner_p) = (
        projection_for(visible_ops(), person(2)),
        projection_for(with_hidden_ops(), person(2)),
    );
    let req = IntroductionRequest::new(p3);
    assert_ne!(
        suggest_introductions(&plain_p, &plain, &req),
        suggest_introductions(&owner_p, &owner, &req)
    );
}
