//! Continuance measures: who and which ties hold a connected group together.
//! The graph is treated as undirected, as `betweenness` does, and only the
//! viewer's projection (the `GraphIndex`) is read.

use std::collections::BTreeMap;

use cn_model::{EdgeId, EntityId};
use serde::{Deserialize, Serialize};

use crate::index::{Adjacency, GraphIndex};

/// A person whose removal splits their connected group.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArticulationPoint {
    pub entity: EntityId,
    /// How many parts the component falls into without this person.
    pub parts: usize,
    /// Node counts of those parts, largest first.
    pub part_sizes: Vec<usize>,
    pub explanation: String,
}

/// A tie whose removal splits its connected group in two.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BridgeTie {
    pub edge: EdgeId,
    pub from: EntityId,
    pub to: EntityId,
    /// Node count of the side that keeps `from`.
    pub from_side: usize,
    /// Node count of the side that keeps `to`.
    pub to_side: usize,
    pub explanation: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Continuance {
    pub articulation_points: BTreeMap<EntityId, ArticulationPoint>,
    pub bridges: BTreeMap<EdgeId, BridgeTie>,
    /// Node count per connected component, keyed like
    /// `EccentricityMeasure::component` (the smallest member id).
    pub components: BTreeMap<EntityId, usize>,
}

/// Articulation points, bridge ties, and component sizes from one iterative
/// Tarjan low-link pass, O(V+E), visiting nodes and neighbors in BTree order.
pub fn continuance(idx: &GraphIndex) -> Continuance {
    let mut walk = Walk::default();
    let mut out = Continuance {
        articulation_points: BTreeMap::new(),
        bridges: BTreeMap::new(),
        components: BTreeMap::new(),
    };
    for &root in &idx.entities {
        if !walk.disc.contains_key(&root) {
            // Roots ascend and earlier components are fully visited, so the
            // root is the component's smallest id.
            let found = walk.component(idx, root);
            finish_component(idx, root, found, &mut out);
        }
    }
    out
}

#[derive(Default)]
struct Walk {
    disc: BTreeMap<EntityId, usize>,
    low: BTreeMap<EntityId, usize>,
    size: BTreeMap<EntityId, usize>,
}

/// What one component's walk found: per node, the sizes of child subtrees
/// that removing it cuts off; per bridge edge, the child endpoint and the
/// size of its subtree.
#[derive(Default)]
struct Found {
    cut_off: BTreeMap<EntityId, Vec<usize>>,
    bridges: Vec<(EdgeId, EntityId, usize)>,
}

struct Frame {
    node: EntityId,
    via: Option<EdgeId>,
    next: usize,
}

impl Walk {
    fn discover(&mut self, node: EntityId) {
        let t = self.disc.len();
        self.disc.insert(node, t);
        self.low.insert(node, t);
        self.size.insert(node, 1);
    }

    fn component(&mut self, idx: &GraphIndex, root: EntityId) -> Found {
        let mut found = Found::default();
        self.discover(root);
        let mut stack = vec![Frame {
            node: root,
            via: None,
            next: 0,
        }];
        while let Some(top) = stack.last_mut() {
            let v = top.node;
            match adjacency(idx, v).get(top.next) {
                Some(adj) => {
                    top.next += 1;
                    // Skip only the tree edge itself, so parallel ties count
                    // as a back edge and are never bridges.
                    if Some(adj.edge) == top.via {
                        continue;
                    }
                    if let Some(&d) = self.disc.get(&adj.to) {
                        self.lower(v, d);
                    } else {
                        self.discover(adj.to);
                        stack.push(Frame {
                            node: adj.to,
                            via: Some(adj.edge),
                            next: 0,
                        });
                    }
                }
                None => {
                    let done = stack.pop().expect("non-empty stack");
                    if let (Some(parent), Some(edge)) = (stack.last(), done.via) {
                        self.retreat(parent.node, v, edge, &mut found);
                    }
                }
            }
        }
        found
    }

    /// Folds finished child `c` (reached over `edge`) into its parent `p`.
    fn retreat(&mut self, p: EntityId, c: EntityId, edge: EdgeId, found: &mut Found) {
        let (low_c, size_c, disc_p) = (self.low[&c], self.size[&c], self.disc[&p]);
        self.lower(p, low_c);
        *self.size.get_mut(&p).expect("discovered") += size_c;
        if low_c >= disc_p {
            found.cut_off.entry(p).or_default().push(size_c);
        }
        if low_c > disc_p {
            found.bridges.push((edge, c, size_c));
        }
    }

    fn lower(&mut self, node: EntityId, value: usize) {
        let low = self.low.get_mut(&node).expect("discovered");
        *low = (*low).min(value);
    }
}

fn adjacency(idx: &GraphIndex, node: EntityId) -> &[Adjacency] {
    idx.adjacency.get(&node).map_or(&[], Vec::as_slice)
}

fn finish_component(idx: &GraphIndex, root: EntityId, found: Found, out: &mut Continuance) {
    let Found { cut_off, bridges } = found;
    let total = cut_off
        .get(&root)
        .map_or(0, |sizes| sizes.iter().sum::<usize>())
        + 1;
    out.components.insert(root, total);
    for (entity, mut part_sizes) in cut_off {
        // The rest of the component stays together through the parent.
        let rest = total - 1 - part_sizes.iter().sum::<usize>();
        if rest > 0 {
            part_sizes.push(rest);
        }
        if part_sizes.len() < 2 {
            continue;
        }
        part_sizes.sort_unstable_by(|a, b| b.cmp(a));
        let point = articulation_point(entity, part_sizes);
        out.articulation_points.insert(entity, point);
    }
    for (edge, child, child_side) in bridges {
        let tie = &idx.edges[&edge];
        let other_side = total - child_side;
        let (from_side, to_side) = if tie.from == child {
            (child_side, other_side)
        } else {
            (other_side, child_side)
        };
        let explanation =
            format!("the only tie between a group of {from_side} and a group of {to_side}");
        out.bridges.insert(
            edge,
            BridgeTie {
                edge,
                from: tie.from,
                to: tie.to,
                from_side,
                to_side,
                explanation,
            },
        );
    }
}

fn articulation_point(entity: EntityId, part_sizes: Vec<usize>) -> ArticulationPoint {
    let parts = part_sizes.len();
    let sizes = part_sizes
        .iter()
        .map(usize::to_string)
        .collect::<Vec<_>>()
        .join(", ");
    ArticulationPoint {
        entity,
        parts,
        explanation: format!("without them, their group splits into {parts} parts of {sizes}"),
        part_sizes,
    }
}
