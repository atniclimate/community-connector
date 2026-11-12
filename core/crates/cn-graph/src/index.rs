use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

use cn_model::{EdgeId, EntityId, KindId};
use cn_perm::{ProjectedEdge, Projection};

use crate::query::{GraphError, PathConstraints};

/// Adjacency index built once per projection revision.
#[derive(Debug, Clone)]
pub struct GraphIndex {
    pub(crate) entities: BTreeSet<EntityId>,
    pub(crate) edges: BTreeMap<EdgeId, ProjectedEdge>,
    pub(crate) adjacency: BTreeMap<EntityId, Vec<Adjacency>>,
}

#[derive(Debug, Clone)]
pub(crate) struct Adjacency {
    pub(crate) to: EntityId,
    pub(crate) edge: EdgeId,
    pub(crate) kind: KindId,
    pub(crate) forward: bool,
    pub(crate) weight: Option<f64>,
}

impl GraphIndex {
    /// Builds an adjacency index from a permission projection.
    pub fn build(p: &Projection) -> Self {
        let entities = p.entities.iter().map(|entity| entity.id).collect();
        let mut idx = Self {
            entities,
            edges: BTreeMap::new(),
            adjacency: BTreeMap::new(),
        };
        for edge in &p.edges {
            idx.insert_edge(edge.clone());
        }
        for neighbors in idx.adjacency.values_mut() {
            neighbors.sort_by(adjacency_order);
        }
        idx
    }

    fn insert_edge(&mut self, edge: ProjectedEdge) {
        self.adjacency
            .entry(edge.from)
            .or_default()
            .push(adjacency(&edge, edge.to, true));
        if !edge.directed {
            self.adjacency
                .entry(edge.to)
                .or_default()
                .push(adjacency(&edge, edge.from, true));
        } else {
            self.adjacency
                .entry(edge.to)
                .or_default()
                .push(adjacency(&edge, edge.from, false));
        }
        self.edges.insert(edge.id, edge);
    }
}

/// All adjacency entries for a node, ignoring `forward`/direction - used by
/// measures defined as undirected (betweenness, eccentricity).
pub(crate) fn all_neighbors(idx: &GraphIndex, node: EntityId) -> impl Iterator<Item = &Adjacency> {
    idx.adjacency.get(&node).into_iter().flatten()
}

pub(crate) fn neighbors<'a>(
    idx: &'a GraphIndex,
    node: EntityId,
    c: &'a PathConstraints,
) -> impl Iterator<Item = &'a Adjacency> + 'a {
    idx.adjacency
        .get(&node)
        .into_iter()
        .flatten()
        .filter(move |adj| edge_allowed(adj, c))
}

fn edge_allowed(adj: &Adjacency, c: &PathConstraints) -> bool {
    if c.respect_direction && !adj.forward {
        return false;
    }
    c.allowed_edge_kinds
        .as_ref()
        .is_none_or(|kinds| kinds.contains(&adj.kind))
}

pub(crate) fn require_entity(idx: &GraphIndex, entity: EntityId) -> Result<(), GraphError> {
    if idx.entities.contains(&entity) {
        Ok(())
    } else {
        Err(GraphError::NotFound)
    }
}

fn adjacency(edge: &ProjectedEdge, to: EntityId, forward: bool) -> Adjacency {
    Adjacency {
        to,
        edge: edge.id,
        kind: edge.kind.clone(),
        forward,
        weight: edge.weight,
    }
}

fn adjacency_order(a: &Adjacency, b: &Adjacency) -> Ordering {
    (a.to, a.edge, &a.kind, a.forward).cmp(&(b.to, b.edge, &b.kind, b.forward))
}
