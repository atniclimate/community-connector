//! Weighted shortest paths under `max_hops`: the audit regression plus a
//! cross-check against exhaustive simple-path enumeration on small graphs
//! generated from fixed seeds (no system entropy).

use std::cmp::Ordering;
use std::collections::BTreeMap;
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

fn edge(n: u128, from: u128, to: u128, directed: bool, weight: Option<f64>) -> ProjectedEdge {
    ProjectedEdge {
        id: uuid(200 + n),
        kind: KindId::new("link").expect("valid kind"),
        from: node(from),
        to: node(to),
        directed,
        weight,
        attributes: BTreeMap::new(),
    }
}

fn projection(nodes: u128, edges: Vec<ProjectedEdge>) -> Projection {
    Projection {
        group_id: uuid(10),
        viewer_fingerprint: "synthetic-viewer".to_string(),
        revision: 1,
        entities: (1..=nodes)
            .map(|n| ProjectedEntity {
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

fn weighted(max_hops: Option<usize>, respect_direction: bool) -> PathConstraints {
    PathConstraints {
        allowed_edge_kinds: None,
        max_hops,
        weighted: true,
        respect_direction,
    }
}

fn cost(e: &ProjectedEdge) -> f64 {
    e.weight.map_or(1.0, |w| 1.0 / w.max(f64::EPSILON))
}

#[test]
fn cheaper_long_prefix_does_not_hide_a_route_that_fits_max_hops() {
    // A=1 B=2 C=3 D=4. A-B-C costs 0.2 over 2 hops, A-C costs 1.0, C-D 0.1.
    let p = projection(
        4,
        vec![
            edge(1, 1, 2, false, Some(10.0)),
            edge(2, 2, 3, false, Some(10.0)),
            edge(3, 1, 3, false, Some(1.0)),
            edge(4, 3, 4, false, Some(10.0)),
        ],
    );
    let idx = GraphIndex::build(&p);
    let path = shortest_path(&idx, node(1), node(4), &weighted(Some(2), false))
        .expect("query")
        .expect("A-C-D fits two hops");
    assert_eq!(path.nodes, vec![node(1), node(3), node(4)]);
    assert_eq!(path.cost, 1.0 + 0.1);
    let unbounded = shortest_path(&idx, node(1), node(4), &weighted(None, false))
        .expect("query")
        .expect("path");
    assert_eq!(unbounded.nodes, vec![node(1), node(2), node(3), node(4)]);
}

/// Edges usable from `at` as (edge, next node), per the direction rule.
fn steps<'a>(
    edges: &'a [ProjectedEdge],
    at: EntityId,
    respect_direction: bool,
) -> impl Iterator<Item = (&'a ProjectedEdge, EntityId)> + 'a {
    edges.iter().filter_map(move |e| {
        if e.from == at {
            Some((e, e.to))
        } else if e.to == at && (!e.directed || !respect_direction) {
            Some((e, e.from))
        } else {
            None
        }
    })
}

/// Best (cost, hops) over every simple path; non-negative costs make a
/// simple path optimal, and costs are summed in path order like the search.
fn enumerate_best(
    edges: &[ProjectedEdge],
    path: &mut Vec<EntityId>,
    cost_so_far: f64,
    target: EntityId,
    c: &PathConstraints,
    best: &mut Option<(f64, usize)>,
) {
    let at = *path.last().expect("non-empty path");
    let hops = path.len() - 1;
    if at == target {
        let better = best.is_none_or(|(bc, bh)| {
            cost_so_far.total_cmp(&bc).then(hops.cmp(&bh)) == Ordering::Less
        });
        if better {
            *best = Some((cost_so_far, hops));
        }
        return;
    }
    if c.max_hops.is_some_and(|max| hops >= max) {
        return;
    }
    for (e, next) in steps(edges, at, c.respect_direction) {
        if path.contains(&next) {
            continue;
        }
        path.push(next);
        enumerate_best(edges, path, cost_so_far + cost(e), target, c, best);
        path.pop();
    }
}

fn assert_valid_path(
    path: &Path,
    edges: &[ProjectedEdge],
    from: EntityId,
    to: EntityId,
    c: &PathConstraints,
) {
    assert_eq!(path.nodes.first(), Some(&from));
    assert_eq!(path.nodes.last(), Some(&to));
    assert_eq!(path.nodes.len(), path.edges.len() + 1);
    let mut total = 0.0;
    for (i, edge_id) in path.edges.iter().enumerate() {
        let (e, _) = steps(edges, path.nodes[i], c.respect_direction)
            .find(|(e, next)| e.id == *edge_id && *next == path.nodes[i + 1])
            .expect("path edge joins consecutive nodes in an allowed direction");
        total += cost(e);
    }
    assert_eq!(total, path.cost);
}

/// Deterministic linear congruential generator for test shapes.
struct Lcg(u64);

impl Lcg {
    fn below(&mut self, n: u64) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        (self.0 >> 33) % n
    }
}

fn generated_edges(rng: &mut Lcg, nodes: u64) -> Vec<ProjectedEdge> {
    const WEIGHTS: [Option<f64>; 6] =
        [None, Some(0.5), Some(1.0), Some(2.0), Some(4.0), Some(10.0)];
    let count = 3 + rng.below(7);
    (0..count)
        .filter_map(|n| {
            let from = 1 + rng.below(nodes);
            let to = 1 + rng.below(nodes);
            let directed = rng.below(2) == 0;
            let weight = WEIGHTS[rng.below(WEIGHTS.len() as u64) as usize];
            (from != to).then(|| {
                edge(
                    u128::from(n),
                    u128::from(from),
                    u128::from(to),
                    directed,
                    weight,
                )
            })
        })
        .collect()
}

#[test]
fn weighted_paths_match_exhaustive_enumeration_on_generated_graphs() {
    let mut checked = 0;
    for seed in 1..=150_u64 {
        let mut rng = Lcg(seed);
        let nodes = 4 + rng.below(3);
        let edges = generated_edges(&mut rng, nodes);
        let idx = GraphIndex::build(&projection(u128::from(nodes), edges.clone()));
        for max_hops in [None, Some(1), Some(2), Some(3), Some(4)] {
            for respect_direction in [false, true] {
                let c = weighted(max_hops, respect_direction);
                for a in 1..=u128::from(nodes) {
                    for b in (1..=u128::from(nodes)).filter(|&b| b != a) {
                        let (from, to) = (node(a), node(b));
                        let mut best = None;
                        enumerate_best(&edges, &mut vec![from], 0.0, to, &c, &mut best);
                        let found = shortest_path(&idx, from, to, &c).expect("query");
                        match (best, found) {
                            (None, None) => {}
                            (Some((cost, hops)), Some(path)) => {
                                assert_valid_path(&path, &edges, from, to, &c);
                                assert_eq!(
                                    (path.cost, path.edges.len()),
                                    (cost, hops),
                                    "seed {seed}"
                                );
                                checked += 1;
                            }
                            (best, found) => {
                                panic!("seed {seed}: expected {best:?}, got {found:?}")
                            }
                        }
                    }
                }
            }
        }
    }
    assert!(
        checked > 1000,
        "generator produced too few reachable pairs: {checked}"
    );
}
