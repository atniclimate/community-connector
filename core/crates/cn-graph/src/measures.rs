use std::collections::{BTreeMap, BTreeSet, VecDeque};

use cn_model::{EntityId, KindId};
use serde::{Deserialize, Serialize};

use crate::index::{GraphIndex, all_neighbors, require_entity};
use crate::query::GraphError;

pub fn degrees(idx: &GraphIndex) -> BTreeMap<EntityId, usize> {
    let mut degrees = idx
        .entities
        .iter()
        .map(|entity| (*entity, 0_usize))
        .collect::<BTreeMap<_, _>>();
    for edge in idx.edges.values() {
        increment_degree(&mut degrees, edge.from);
        if edge.to != edge.from {
            increment_degree(&mut degrees, edge.to);
        }
    }
    degrees
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DegreeMeasure {
    pub entity: EntityId,
    pub degree: usize,
    pub single_tie: bool,
    pub explanation: String,
}

/// Degree plus the single-tie flag (degree == 1), each with an explanation.
pub fn degree_measures(idx: &GraphIndex) -> BTreeMap<EntityId, DegreeMeasure> {
    degrees(idx)
        .into_iter()
        .map(|(entity, degree)| {
            let single_tie = degree == 1;
            let explanation = if degree == 0 {
                "has no connections".to_string()
            } else if single_tie {
                "connected to 1 other person".to_string()
            } else {
                format!("connected to {degree} others")
            };
            (
                entity,
                DegreeMeasure {
                    entity,
                    degree,
                    single_tie,
                    explanation,
                },
            )
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JaccardMeasure {
    pub a: EntityId,
    pub b: EntityId,
    pub shared: usize,
    pub union: usize,
    pub value: f64,
    pub explanation: String,
}

/// Jaccard index over committee membership (`member_of`-style edges): the
/// fraction of the two entities' COMBINED committees that they share.
pub fn shared_committee_jaccard(
    idx: &GraphIndex,
    a: EntityId,
    b: EntityId,
    membership_kind: &KindId,
) -> Result<JaccardMeasure, GraphError> {
    require_entity(idx, a)?;
    require_entity(idx, b)?;
    let committees_a = committees_of(idx, a, membership_kind);
    let committees_b = committees_of(idx, b, membership_kind);
    let shared = committees_a.intersection(&committees_b).count();
    let union = committees_a.union(&committees_b).count();
    let value = if union == 0 {
        0.0
    } else {
        shared as f64 / union as f64
    };
    let explanation = if union == 0 {
        "belongs to no shared committees".to_string()
    } else {
        format!("shares {shared} of {union} committees")
    };
    Ok(JaccardMeasure {
        a,
        b,
        shared,
        union,
        value,
        explanation,
    })
}

fn committees_of(
    idx: &GraphIndex,
    entity: EntityId,
    membership_kind: &KindId,
) -> BTreeSet<EntityId> {
    idx.adjacency
        .get(&entity)
        .into_iter()
        .flatten()
        .filter(|adj| adj.forward && &adj.kind == membership_kind)
        .map(|adj| adj.to)
        .collect()
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BetweennessMeasure {
    pub entity: EntityId,
    pub value: f64,
    pub explanation: String,
}

/// Exact Brandes' betweenness centrality over unweighted shortest paths,
/// traversing every adjacency entry regardless of edge direction (I2: the
/// index already carries only the permission-filtered projection's edges).
/// Normalized by (n-1)(n-2); zero for n <= 2.
pub fn betweenness(idx: &GraphIndex) -> BTreeMap<EntityId, BetweennessMeasure> {
    let nodes: Vec<EntityId> = idx.entities.iter().copied().collect();
    let mut centrality: BTreeMap<EntityId, f64> = nodes.iter().map(|&v| (v, 0.0)).collect();
    for &s in &nodes {
        let paths = single_source_paths(idx, &nodes, s);
        accumulate_dependencies(s, &nodes, paths, &mut centrality);
    }
    normalize_betweenness(nodes, &centrality)
}

/// One Brandes BFS from `source`: visit order, shortest-path predecessors,
/// and shortest-path counts.
struct SourcePaths {
    stack: Vec<EntityId>,
    predecessors: BTreeMap<EntityId, Vec<EntityId>>,
    sigma: BTreeMap<EntityId, f64>,
}

fn single_source_paths(idx: &GraphIndex, nodes: &[EntityId], s: EntityId) -> SourcePaths {
    let mut stack = Vec::new();
    let mut predecessors: BTreeMap<EntityId, Vec<EntityId>> =
        nodes.iter().map(|&v| (v, Vec::new())).collect();
    let mut sigma: BTreeMap<EntityId, f64> = nodes.iter().map(|&v| (v, 0.0)).collect();
    let mut dist: BTreeMap<EntityId, i64> = nodes.iter().map(|&v| (v, -1)).collect();
    sigma.insert(s, 1.0);
    dist.insert(s, 0);
    let mut queue = VecDeque::from([s]);
    while let Some(v) = queue.pop_front() {
        stack.push(v);
        let dv = dist[&v];
        let sv = sigma[&v];
        for adj in all_neighbors(idx, v) {
            let w = adj.to;
            if dist[&w] < 0 {
                dist.insert(w, dv + 1);
                queue.push_back(w);
            }
            if dist[&w] == dv + 1 {
                *sigma.get_mut(&w).expect("known node") += sv;
                predecessors.get_mut(&w).expect("known node").push(v);
            }
        }
    }
    SourcePaths {
        stack,
        predecessors,
        sigma,
    }
}

fn accumulate_dependencies(
    s: EntityId,
    nodes: &[EntityId],
    paths: SourcePaths,
    centrality: &mut BTreeMap<EntityId, f64>,
) {
    let SourcePaths {
        mut stack,
        predecessors,
        sigma,
    } = paths;
    let mut delta: BTreeMap<EntityId, f64> = nodes.iter().map(|&v| (v, 0.0)).collect();
    while let Some(w) = stack.pop() {
        let sigma_w = sigma[&w];
        let delta_w = delta[&w];
        for &v in &predecessors[&w] {
            let contribution = (sigma[&v] / sigma_w) * (1.0 + delta_w);
            *delta.get_mut(&v).expect("known node") += contribution;
        }
        if w != s {
            *centrality.get_mut(&w).expect("known node") += delta_w;
        }
    }
}

fn normalize_betweenness(
    nodes: Vec<EntityId>,
    centrality: &BTreeMap<EntityId, f64>,
) -> BTreeMap<EntityId, BetweennessMeasure> {
    let n = nodes.len();
    let normalization = if n > 2 {
        ((n - 1) * (n - 2)) as f64
    } else {
        0.0
    };
    nodes
        .into_iter()
        .map(|entity| {
            let raw = centrality[&entity];
            let value = if normalization > 0.0 {
                raw / normalization
            } else {
                0.0
            };
            let explanation = format!(
                "sits on {:.1}% of shortest paths between other people",
                value * 100.0
            );
            (
                entity,
                BetweennessMeasure {
                    entity,
                    value,
                    explanation,
                },
            )
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EccentricityMeasure {
    pub entity: EntityId,
    pub steps: usize,
    pub component: EntityId,
    pub explanation: String,
}

/// BFS eccentricity per node (steps to the farthest reachable node),
/// undirected traversal like `betweenness`. `component` is the smallest
/// `EntityId` reachable from this node, so two nodes share `component` iff
/// they are mutually reachable - this is how disconnected nodes report their
/// own component instead of being silently folded into the whole graph.
pub fn eccentricity(idx: &GraphIndex) -> BTreeMap<EntityId, EccentricityMeasure> {
    let components = connected_components(idx);
    idx.entities
        .iter()
        .map(|&node| {
            let mut dist: BTreeMap<EntityId, usize> = BTreeMap::from([(node, 0)]);
            let mut queue = VecDeque::from([node]);
            let mut steps = 0;
            while let Some(v) = queue.pop_front() {
                let d = dist[&v];
                for adj in all_neighbors(idx, v) {
                    if let std::collections::btree_map::Entry::Vacant(entry) = dist.entry(adj.to) {
                        entry.insert(d + 1);
                        steps = steps.max(d + 1);
                        queue.push_back(adj.to);
                    }
                }
            }
            let explanation = format!("{steps} steps from the farthest person");
            (
                node,
                EccentricityMeasure {
                    entity: node,
                    steps,
                    component: components[&node],
                    explanation,
                },
            )
        })
        .collect()
}

fn connected_components(idx: &GraphIndex) -> BTreeMap<EntityId, EntityId> {
    let mut roots: BTreeMap<EntityId, EntityId> = BTreeMap::new();
    for &start in &idx.entities {
        if roots.contains_key(&start) {
            continue;
        }
        let mut members = vec![start];
        let mut seen = BTreeSet::from([start]);
        let mut queue = VecDeque::from([start]);
        while let Some(v) = queue.pop_front() {
            for adj in all_neighbors(idx, v) {
                if seen.insert(adj.to) {
                    members.push(adj.to);
                    queue.push_back(adj.to);
                }
            }
        }
        let root = *members.iter().min().expect("at least the start node");
        for member in members {
            roots.insert(member, root);
        }
    }
    roots
}

fn increment_degree(degrees: &mut BTreeMap<EntityId, usize>, entity: EntityId) {
    if let Some(degree) = degrees.get_mut(&entity) {
        *degree = degree.saturating_add(1);
    }
}
