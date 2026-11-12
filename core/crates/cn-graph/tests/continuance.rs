//! Continuance measures (articulation points, bridge ties, component sizes)
//! on hand-checked shapes plus a brute-force cross-check on small graphs
//! generated from fixed seeds (no system entropy).

use std::collections::{BTreeMap, BTreeSet};
use std::str::FromStr;

use cn_graph::*;
use cn_model::*;
use cn_perm::{ProjectedEdge, ProjectedEntity, Projection};

fn uuid<T: FromStr>(n: u128) -> T
where
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

fn node(n: u128) -> EntityId {
    uuid(100 + n)
}

fn edge_id(n: u128) -> EdgeId {
    uuid(10_000 + n)
}

fn tie(n: u128, from: u128, to: u128, directed: bool) -> ProjectedEdge {
    ProjectedEdge {
        id: edge_id(n),
        kind: KindId::new("link").expect("valid kind"),
        from: node(from),
        to: node(to),
        directed,
        weight: None,
        attributes: BTreeMap::new(),
    }
}

fn projection(nodes: &[u128], edges: Vec<ProjectedEdge>) -> Projection {
    Projection {
        group_id: uuid(10),
        viewer_fingerprint: "synthetic-viewer".to_string(),
        revision: 1,
        entities: nodes
            .iter()
            .map(|&n| ProjectedEntity {
                id: node(n),
                kind: KindId::new("person").expect("valid kind"),
                owner_is_viewer: false,
                attributes: BTreeMap::new(),
            })
            .collect(),
        edges,
        stories: Vec::new(),
    }
}

/// Undirected ties numbered in order, over nodes 1..=n.
fn graph(n: u128, pairs: &[(u128, u128)]) -> Continuance {
    let nodes: Vec<u128> = (1..=n).collect();
    let edges = pairs
        .iter()
        .enumerate()
        .map(|(i, &(a, b))| tie(i as u128 + 1, a, b, false))
        .collect();
    continuance(&GraphIndex::build(&projection(&nodes, edges)))
}

fn points(c: &Continuance) -> BTreeMap<EntityId, Vec<usize>> {
    c.articulation_points
        .iter()
        .map(|(id, p)| {
            assert_eq!(p.parts, p.part_sizes.len());
            assert!(!p.explanation.is_empty());
            (*id, p.part_sizes.clone())
        })
        .collect()
}

fn bridges(c: &Continuance) -> BTreeMap<EdgeId, (usize, usize)> {
    c.bridges
        .iter()
        .map(|(id, b)| {
            assert!(!b.explanation.is_empty());
            (*id, (b.from_side, b.to_side))
        })
        .collect()
}

#[test]
fn path_has_inner_cut_points_and_every_tie_is_a_bridge() {
    let c = graph(4, &[(1, 2), (2, 3), (3, 4)]);
    assert_eq!(
        points(&c),
        BTreeMap::from([(node(2), vec![2, 1]), (node(3), vec![2, 1])])
    );
    assert_eq!(
        bridges(&c),
        BTreeMap::from([
            (edge_id(1), (1, 3)),
            (edge_id(2), (2, 2)),
            (edge_id(3), (3, 1)),
        ])
    );
    assert_eq!(c.components, BTreeMap::from([(node(1), 4)]));
    let p = &c.articulation_points[&node(2)];
    assert_eq!(
        p.explanation,
        "without them, their group splits into 2 parts of 2, 1"
    );
    assert_eq!(
        c.bridges[&edge_id(1)].explanation,
        "the only tie between a group of 1 and a group of 3"
    );
}

#[test]
fn star_center_splits_into_every_leaf() {
    let c = graph(5, &[(1, 2), (1, 3), (1, 4), (1, 5)]);
    assert_eq!(points(&c), BTreeMap::from([(node(1), vec![1, 1, 1, 1])]));
    assert_eq!(bridges(&c).len(), 4);
    assert!(bridges(&c).values().all(|&sides| sides == (4, 1)));
}

#[test]
fn cycle_has_no_cut_points_or_bridges() {
    let c = graph(4, &[(1, 2), (2, 3), (3, 4), (4, 1)]);
    assert!(c.articulation_points.is_empty());
    assert!(c.bridges.is_empty());
    assert_eq!(c.components, BTreeMap::from([(node(1), 4)]));
}

#[test]
fn two_cliques_joined_through_one_person() {
    // {1,2,3} and {5,6,7} are triangles; 4 ties to 3 and to 5 only.
    let c = graph(
        7,
        &[
            (1, 2),
            (2, 3),
            (1, 3),
            (3, 4),
            (4, 5),
            (5, 6),
            (6, 7),
            (5, 7),
        ],
    );
    assert_eq!(
        points(&c),
        BTreeMap::from([
            (node(3), vec![4, 2]),
            (node(4), vec![3, 3]),
            (node(5), vec![4, 2]),
        ])
    );
    // Both of person 4's ties are bridges, and they are the only ones.
    assert_eq!(
        bridges(&c),
        BTreeMap::from([(edge_id(4), (3, 4)), (edge_id(5), (4, 3))])
    );
}

#[test]
fn tree_every_tie_is_a_bridge() {
    let c = graph(6, &[(1, 2), (1, 3), (2, 4), (2, 5), (3, 6)]);
    assert_eq!(
        points(&c),
        BTreeMap::from([
            (node(1), vec![3, 2]),
            (node(2), vec![3, 1, 1]),
            (node(3), vec![4, 1]),
        ])
    );
    assert_eq!(
        bridges(&c),
        BTreeMap::from([
            (edge_id(1), (3, 3)),
            (edge_id(2), (4, 2)),
            (edge_id(3), (5, 1)),
            (edge_id(4), (5, 1)),
            (edge_id(5), (5, 1)),
        ])
    );
}

#[test]
fn disconnected_components_are_counted_separately() {
    // Path 1-2-3, triangle 4-5-6, and isolated 7.
    let c = graph(7, &[(1, 2), (2, 3), (4, 5), (5, 6), (6, 4)]);
    assert_eq!(
        c.components,
        BTreeMap::from([(node(1), 3), (node(4), 3), (node(7), 1)])
    );
    assert_eq!(points(&c), BTreeMap::from([(node(2), vec![1, 1])]));
    assert_eq!(bridges(&c).len(), 2);
}

#[test]
fn parallel_ties_self_loops_and_direction_do_not_make_bridges() {
    let nodes = [1, 2, 3];
    let edges = vec![
        tie(1, 1, 2, true),
        tie(2, 2, 1, true),
        tie(3, 2, 3, true),
        tie(4, 3, 3, false),
    ];
    let c = continuance(&GraphIndex::build(&projection(&nodes, edges)));
    assert_eq!(bridges(&c), BTreeMap::from([(edge_id(3), (2, 1))]));
    assert_eq!(points(&c), BTreeMap::from([(node(2), vec![1, 1])]));
}

#[test]
fn results_do_not_depend_on_insertion_order() {
    let pairs = [
        (1, 2),
        (2, 3),
        (1, 3),
        (3, 4),
        (4, 5),
        (5, 6),
        (6, 4),
        (7, 8),
    ];
    let edges: Vec<ProjectedEdge> = pairs
        .iter()
        .enumerate()
        .map(|(i, &(a, b))| tie(i as u128 + 1, a, b, false))
        .collect();
    let nodes: Vec<u128> = (1..=9).collect();
    let forward = continuance(&GraphIndex::build(&projection(&nodes, edges.clone())));
    let mut reversed_nodes = nodes.clone();
    reversed_nodes.reverse();
    let mut shuffled = edges;
    shuffled.rotate_left(3);
    shuffled.reverse();
    let backward = continuance(&GraphIndex::build(&projection(&reversed_nodes, shuffled)));
    assert_eq!(forward, backward);
    assert_eq!(
        format!("{forward:?}"),
        format!("{backward:?}"),
        "identical output byte for byte"
    );
}

/// Component sizes of `pairs` over `nodes`, ignoring `skip_node`/`skip_edge`.
fn brute_parts(
    nodes: &BTreeSet<u128>,
    pairs: &[(u128, u128)],
    skip_node: Option<u128>,
    skip_edge: Option<usize>,
) -> Vec<usize> {
    let mut seen = BTreeSet::new();
    let mut sizes = Vec::new();
    for &start in nodes {
        if Some(start) == skip_node || !seen.insert(start) {
            continue;
        }
        let (mut stack, mut size) = (vec![start], 0);
        while let Some(v) = stack.pop() {
            size += 1;
            for (i, &(a, b)) in pairs.iter().enumerate() {
                let other = if a == v {
                    b
                } else if b == v {
                    a
                } else {
                    continue;
                };
                if Some(i) != skip_edge && Some(other) != skip_node && seen.insert(other) {
                    stack.push(other);
                }
            }
        }
        sizes.push(size);
    }
    sizes.sort_unstable_by(|a, b| b.cmp(a));
    sizes
}

#[test]
fn matches_brute_force_removal_on_seeded_graphs() {
    let mut seed: u64 = 0x5eed_c0de;
    let mut next = |bound: u64| {
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (seed >> 33) % bound
    };
    for _ in 0..200 {
        let n = 2 + next(8) as u128;
        let nodes: BTreeSet<u128> = (1..=n).collect();
        let pairs: Vec<(u128, u128)> = (0..next(12))
            .map(|_| (1 + next(n as u64) as u128, 1 + next(n as u64) as u128))
            .collect();
        let c = graph(n, &pairs);
        let base = brute_parts(&nodes, &pairs, None, None);
        for &v in &nodes {
            let comp_of_v = base.len();
            let without = brute_parts(&nodes, &pairs, Some(v), None);
            // v's component split into (without.len() - comp_of_v + 1) parts.
            let split = without.len() + 1 - comp_of_v;
            assert_eq!(
                c.articulation_points.contains_key(&node(v)),
                split >= 2,
                "node {v} in {pairs:?}"
            );
        }
        for i in 0..pairs.len() {
            let without = brute_parts(&nodes, &pairs, None, Some(i));
            assert_eq!(
                c.bridges.contains_key(&edge_id(i as u128 + 1)),
                without.len() > base.len(),
                "edge {i} in {pairs:?}"
            );
        }
        assert_eq!(c.components.values().sum::<usize>(), nodes.len());
    }
}
