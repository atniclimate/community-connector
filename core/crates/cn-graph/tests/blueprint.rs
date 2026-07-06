#![allow(clippy::too_many_lines)]

//! I5 note: this acceptance module covers the cn-graph blueprint obligations
//! together because the projection fixture is shared across query behaviors.

use std::collections::{BTreeMap, BTreeSet};
use std::str::FromStr;

use cn_graph::*;
use cn_model::*;
use cn_perm::{ProjectedEdge, ProjectedEntity, Projection};

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

fn entity_id(n: u128) -> EntityId {
    id(100 + n)
}

fn edge_id(n: u128) -> EdgeId {
    id(200 + n)
}

fn group_id() -> GroupId {
    id(10)
}

fn kind(value: &str) -> KindId {
    KindId::new(value).expect("valid kind")
}

fn attr(value: &str) -> AttrId {
    AttrId::new(value).expect("valid attr")
}

fn projection(entities: Vec<ProjectedEntity>, edges: Vec<ProjectedEdge>) -> Projection {
    Projection {
        group_id: group_id(),
        viewer_fingerprint: "synthetic-viewer".to_string(),
        revision: 1,
        entities,
        edges,
        stories: Vec::new(),
    }
}

fn entity(n: u128, kind_name: &str) -> ProjectedEntity {
    ProjectedEntity {
        id: entity_id(n),
        kind: kind(kind_name),
        owner_is_viewer: false,
        attributes: BTreeMap::new(),
    }
}

fn edge(
    n: u128,
    from: u128,
    to: u128,
    kind_name: &str,
    directed: bool,
    weight: Option<f64>,
) -> ProjectedEdge {
    ProjectedEdge {
        id: edge_id(n),
        kind: kind(kind_name),
        from: entity_id(from),
        to: entity_id(to),
        directed,
        weight,
        attributes: BTreeMap::new(),
    }
}

fn path_ids(path: Option<Path>) -> Vec<EntityId> {
    path.expect("path found").nodes
}

#[test]
fn need_to_solution_routing_and_weighted_preference() {
    let p = projection(
        vec![
            entity(1, "need"),
            entity(2, "solution"),
            entity(3, "skill"),
            entity(4, "person"),
            entity(5, "detour"),
        ],
        vec![
            edge(1, 1, 2, "need_met_by", true, Some(2.0)),
            edge(2, 2, 3, "has_skill", true, Some(2.0)),
            edge(3, 3, 4, "held_by", true, Some(2.0)),
            edge(4, 1, 5, "mentions", true, Some(100.0)),
            edge(5, 5, 4, "mentions", true, Some(100.0)),
            edge(6, 1, 3, "need_met_by", true, Some(0.1)),
        ],
    );
    let idx = GraphIndex::build(&p);
    let constrained = PathConstraints {
        allowed_edge_kinds: Some(vec![
            kind("need_met_by"),
            kind("has_skill"),
            kind("held_by"),
        ]),
        max_hops: None,
        weighted: false,
        respect_direction: true,
    };
    assert_eq!(
        path_ids(shortest_path(&idx, entity_id(1), entity_id(4), &constrained).expect("query")),
        vec![entity_id(1), entity_id(3), entity_id(4)]
    );
    let excluded = PathConstraints {
        allowed_edge_kinds: Some(vec![kind("mentions")]),
        ..constrained.clone()
    };
    assert_eq!(
        shortest_path(&idx, entity_id(2), entity_id(4), &excluded).expect("query"),
        None
    );

    let weighted = PathConstraints {
        weighted: true,
        allowed_edge_kinds: Some(vec![
            kind("need_met_by"),
            kind("has_skill"),
            kind("held_by"),
        ]),
        ..constrained
    };
    let path = shortest_path(&idx, entity_id(1), entity_id(4), &weighted)
        .expect("query")
        .expect("path");
    assert_eq!(
        path.nodes,
        vec![entity_id(1), entity_id(2), entity_id(3), entity_id(4)]
    );
    assert_eq!(path.cost, 1.5);
}

#[test]
fn direction_changes_reachability() {
    let p = projection(
        vec![entity(1, "person"), entity(2, "person")],
        vec![edge(1, 1, 2, "knows", true, None)],
    );
    let idx = GraphIndex::build(&p);
    let directed = PathConstraints {
        respect_direction: true,
        ..PathConstraints::default()
    };
    let undirected = PathConstraints {
        respect_direction: false,
        ..PathConstraints::default()
    };
    assert_eq!(
        shortest_path(&idx, entity_id(2), entity_id(1), &directed).expect("query"),
        None
    );
    assert_eq!(
        path_ids(shortest_path(&idx, entity_id(2), entity_id(1), &undirected).expect("query")),
        vec![entity_id(2), entity_id(1)]
    );
}

#[test]
fn max_hops_and_neighborhood_layers_are_bounded_and_deduped() {
    let p = projection(
        vec![
            entity(1, "node"),
            entity(2, "node"),
            entity(3, "node"),
            entity(4, "node"),
        ],
        vec![
            edge(1, 1, 2, "link", false, None),
            edge(2, 2, 3, "link", false, None),
            edge(3, 1, 3, "link", false, None),
            edge(4, 3, 4, "link", false, None),
            edge(5, 2, 4, "link", false, None),
        ],
    );
    let idx = GraphIndex::build(&p);
    let limited = PathConstraints {
        max_hops: Some(1),
        ..PathConstraints::default()
    };
    assert_eq!(
        shortest_path(&idx, entity_id(1), entity_id(4), &limited).expect("query"),
        None
    );
    assert_eq!(
        neighborhood(&idx, entity_id(1), 0, &PathConstraints::default()).expect("query"),
        Neighborhood {
            center: entity_id(1),
            layers: Vec::new()
        }
    );
    let n = neighborhood(&idx, entity_id(1), 3, &PathConstraints::default()).expect("query");
    assert_eq!(n.layers[0], vec![entity_id(2), entity_id(3)]);
    assert_eq!(n.layers[1], vec![entity_id(4)]);
    assert_eq!(n.layers.len(), 2);
}

#[test]
fn absent_and_hidden_ids_have_the_same_not_found_error() {
    let p = projection(vec![entity(1, "node")], Vec::new());
    let idx = GraphIndex::build(&p);
    let hidden = entity_id(2);
    let missing = entity_id(99);
    assert_eq!(
        shortest_path(&idx, entity_id(1), hidden, &PathConstraints::default()),
        Err(GraphError::NotFound)
    );
    assert_eq!(
        shortest_path(&idx, entity_id(1), missing, &PathConstraints::default()),
        Err(GraphError::NotFound)
    );
    assert_eq!(
        neighborhood(&idx, hidden, 1, &PathConstraints::default()),
        Err(GraphError::NotFound)
    );
}

#[test]
fn deterministic_outputs_and_degree_ordering() {
    let p = projection(
        vec![entity(3, "node"), entity(1, "node"), entity(2, "node")],
        vec![
            edge(2, 1, 3, "link", false, None),
            edge(1, 1, 2, "link", false, None),
        ],
    );
    let first = GraphIndex::build(&p);
    let second = GraphIndex::build(&p);
    let constraints = PathConstraints::default();
    assert_eq!(
        shortest_path(&first, entity_id(1), entity_id(3), &constraints).expect("query"),
        shortest_path(&second, entity_id(1), entity_id(3), &constraints).expect("query")
    );
    assert_eq!(
        neighborhood(&first, entity_id(1), 2, &constraints).expect("query"),
        neighborhood(&second, entity_id(1), 2, &constraints).expect("query")
    );
    assert_eq!(
        degrees(&first),
        BTreeMap::from([(entity_id(1), 2), (entity_id(2), 1), (entity_id(3), 1)])
    );
}

#[test]
fn search_matches_projected_attribute_values_with_stable_ordering() {
    let mut first = entity(1, "person");
    first.attributes.insert(
        attr("name"),
        AttributeValue::Text("Synthetic Harbor Guide".to_string()),
    );
    first.attributes.insert(
        attr("tags"),
        AttributeValue::Tags(BTreeSet::from([
            "fisheries".to_string(),
            "harbor".to_string(),
        ])),
    );
    let mut second = entity(2, "place");
    second.attributes.insert(
        attr("role"),
        AttributeValue::Enum("Harbor coordinator".to_string()),
    );
    let mut third = entity(3, "person");
    third.attributes.insert(
        attr("site"),
        AttributeValue::Link(
            LinkValue::new("https://example.test/harbor", Some(LinkFormat::Url))
                .expect("valid url"),
        ),
    );
    let p = projection(vec![third, second, first], Vec::new());
    let q = SearchQuery {
        text: "HARBOR".to_string(),
        kinds: None,
        limit: 10,
    };
    let hits = search(&p, &q);
    assert_eq!(hits.len(), 4);
    assert_eq!(hits[0].entity, entity_id(1));
    assert_eq!(hits[0].matched_attr, attr("tags"));
    assert_eq!(hits[1].entity, entity_id(2));
    assert_eq!(hits[1].matched_attr, attr("role"));
    assert_eq!(hits[2].entity, entity_id(1));
    assert_eq!(hits[2].matched_attr, attr("name"));

    let people = search(
        &p,
        &SearchQuery {
            kinds: Some(vec![kind("person")]),
            limit: 1,
            ..q.clone()
        },
    );
    assert_eq!(people.len(), 1);
    assert_eq!(people[0].entity, entity_id(1));
    assert!(
        search(
            &p,
            &SearchQuery {
                text: " ".to_string(),
                ..q
            }
        )
        .is_empty()
    );
}

#[test]
fn search_snippet_truncates_on_char_boundary() {
    let mut entity = entity(1, "person");
    let long = format!("{}harbor", "a".repeat(100));
    entity
        .attributes
        .insert(attr("bio"), AttributeValue::Text(long));
    let p = projection(vec![entity], Vec::new());
    let hits = search(
        &p,
        &SearchQuery {
            text: "harbor".to_string(),
            kinds: None,
            limit: 10,
        },
    );
    assert_eq!(hits[0].snippet.chars().count(), 80);
}
