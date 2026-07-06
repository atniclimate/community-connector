use std::cmp::{Ordering, Reverse};
use std::collections::{BTreeMap, BTreeSet, BinaryHeap, VecDeque};

use cn_model::{AttrId, AttributeValue, EdgeId, EntityId, KindId};
use cn_perm::{ProjectedEdge, Projection};
use serde::{Deserialize, Serialize};

/// Adjacency index built once per projection revision.
#[derive(Debug, Clone)]
pub struct GraphIndex {
    entities: BTreeSet<EntityId>,
    edges: BTreeMap<EdgeId, ProjectedEdge>,
    adjacency: BTreeMap<EntityId, Vec<Adjacency>>,
}

#[derive(Debug, Clone)]
struct Adjacency {
    to: EntityId,
    edge: EdgeId,
    kind: KindId,
    forward: bool,
    weight: Option<f64>,
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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PathConstraints {
    pub allowed_edge_kinds: Option<Vec<KindId>>,
    pub max_hops: Option<usize>,
    pub weighted: bool,
    pub respect_direction: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Path {
    pub nodes: Vec<EntityId>,
    pub edges: Vec<EdgeId>,
    pub cost: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Neighborhood {
    pub center: EntityId,
    pub layers: Vec<Vec<EntityId>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SearchQuery {
    pub text: String,
    pub kinds: Option<Vec<KindId>>,
    pub limit: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchHit {
    pub entity: EntityId,
    pub kind: KindId,
    pub matched_attr: AttrId,
    pub snippet: String,
}

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum GraphError {
    #[error("not found")]
    NotFound,
}

pub fn shortest_path(
    idx: &GraphIndex,
    from: EntityId,
    to: EntityId,
    c: &PathConstraints,
) -> Result<Option<Path>, GraphError> {
    require_entity(idx, from)?;
    require_entity(idx, to)?;
    if from == to {
        return Ok(Some(Path {
            nodes: vec![from],
            edges: Vec::new(),
            cost: 0.0,
        }));
    }
    if c.weighted {
        Ok(weighted_path(idx, from, to, c))
    } else {
        Ok(unweighted_path(idx, from, to, c))
    }
}

pub fn neighborhood(
    idx: &GraphIndex,
    center: EntityId,
    hops: usize,
    c: &PathConstraints,
) -> Result<Neighborhood, GraphError> {
    require_entity(idx, center)?;
    let limit = c.max_hops.map_or(hops, |max| max.min(hops));
    let mut seen = BTreeSet::from([center]);
    let mut frontier = BTreeSet::from([center]);
    let mut layers = Vec::new();
    for _ in 0..limit {
        let layer = neighborhood_layer(idx, &frontier, &seen, c);
        if layer.is_empty() {
            break;
        }
        seen.extend(layer.iter().copied());
        frontier = layer.iter().copied().collect();
        layers.push(layer);
    }
    Ok(Neighborhood { center, layers })
}

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

pub fn search(p: &Projection, q: &SearchQuery) -> Vec<SearchHit> {
    let needle = q.text.trim().to_lowercase();
    if needle.is_empty() || q.limit == 0 {
        return Vec::new();
    }
    let allowed = q
        .kinds
        .as_ref()
        .map(|kinds| kinds.iter().collect::<BTreeSet<_>>());
    let mut hits = Vec::new();
    for entity in &p.entities {
        if allowed
            .as_ref()
            .is_some_and(|kinds| !kinds.contains(&entity.kind))
        {
            continue;
        }
        collect_entity_hits(entity, &needle, &mut hits);
    }
    hits.sort_by(search_hit_order);
    hits.truncate(q.limit);
    hits.into_iter().map(|hit| hit.hit).collect()
}

fn unweighted_path(
    idx: &GraphIndex,
    from: EntityId,
    to: EntityId,
    c: &PathConstraints,
) -> Option<Path> {
    let mut queue = VecDeque::from([from]);
    let mut previous = BTreeMap::new();
    let mut hops = BTreeMap::from([(from, 0_usize)]);
    while let Some(node) = queue.pop_front() {
        let next_hop = hops.get(&node).copied().unwrap_or(0).saturating_add(1);
        if c.max_hops.is_some_and(|max| next_hop > max) {
            continue;
        }
        for adj in neighbors(idx, node, c) {
            if hops.contains_key(&adj.to) {
                continue;
            }
            previous.insert(adj.to, (node, adj.edge));
            if adj.to == to {
                return Some(reconstruct(from, to, &previous, next_hop as f64));
            }
            hops.insert(adj.to, next_hop);
            queue.push_back(adj.to);
        }
    }
    None
}

fn weighted_path(
    idx: &GraphIndex,
    from: EntityId,
    to: EntityId,
    c: &PathConstraints,
) -> Option<Path> {
    let mut heap = BinaryHeap::from([Reverse(QueueState::new(0.0, 0, from))]);
    let mut best = BTreeMap::from([(from, (0.0, 0_usize))]);
    let mut previous = BTreeMap::new();
    while let Some(Reverse(state)) = heap.pop() {
        if stale_state(&best, state) {
            continue;
        }
        if state.node == to {
            return Some(reconstruct(from, to, &previous, state.cost));
        }
        push_weighted_neighbors(idx, c, state, &mut best, &mut previous, &mut heap);
    }
    None
}

fn push_weighted_neighbors(
    idx: &GraphIndex,
    c: &PathConstraints,
    state: QueueState,
    best: &mut BTreeMap<EntityId, (f64, usize)>,
    previous: &mut BTreeMap<EntityId, (EntityId, EdgeId)>,
    heap: &mut BinaryHeap<Reverse<QueueState>>,
) {
    let next_hops = state.hops.saturating_add(1);
    if c.max_hops.is_some_and(|max| next_hops > max) {
        return;
    }
    for adj in neighbors(idx, state.node, c) {
        let next_cost = state.cost + edge_cost(adj);
        if !is_better(best.get(&adj.to), next_cost, next_hops) {
            continue;
        }
        best.insert(adj.to, (next_cost, next_hops));
        previous.insert(adj.to, (state.node, adj.edge));
        heap.push(Reverse(QueueState::new(next_cost, next_hops, adj.to)));
    }
}

fn reconstruct(
    from: EntityId,
    to: EntityId,
    previous: &BTreeMap<EntityId, (EntityId, EdgeId)>,
    cost: f64,
) -> Path {
    let mut nodes = vec![to];
    let mut edges = Vec::new();
    let mut current = to;
    while current != from {
        if let Some((prev, edge)) = previous.get(&current) {
            edges.push(*edge);
            current = *prev;
            nodes.push(current);
        } else {
            break;
        }
    }
    nodes.reverse();
    edges.reverse();
    Path { nodes, edges, cost }
}

fn neighborhood_layer(
    idx: &GraphIndex,
    frontier: &BTreeSet<EntityId>,
    seen: &BTreeSet<EntityId>,
    c: &PathConstraints,
) -> Vec<EntityId> {
    let mut layer = BTreeSet::new();
    for node in frontier {
        for adj in neighbors(idx, *node, c) {
            if !seen.contains(&adj.to) {
                layer.insert(adj.to);
            }
        }
    }
    layer.into_iter().collect()
}

fn neighbors<'a>(
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

fn collect_entity_hits(
    entity: &cn_perm::ProjectedEntity,
    needle: &str,
    hits: &mut Vec<OrderedHit>,
) {
    for (attr, value) in &entity.attributes {
        for candidate in searchable_values(value) {
            let haystack = candidate.to_lowercase();
            if let Some(position) = haystack.find(needle) {
                hits.push(OrderedHit {
                    position,
                    entity: entity.id,
                    attr: attr.clone(),
                    hit: SearchHit {
                        entity: entity.id,
                        kind: entity.kind.clone(),
                        matched_attr: attr.clone(),
                        snippet: truncate_snippet(&candidate),
                    },
                });
            }
        }
    }
}

fn searchable_values(value: &AttributeValue) -> Vec<String> {
    match value {
        AttributeValue::Text(value) | AttributeValue::Enum(value) => vec![value.clone()],
        AttributeValue::Tags(tags) => tags.iter().cloned().collect(),
        AttributeValue::Link(link) => vec![link.url().to_string()],
        _ => Vec::new(),
    }
}

fn truncate_snippet(value: &str) -> String {
    value.chars().take(80).collect()
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

fn edge_cost(adj: &Adjacency) -> f64 {
    adj.weight
        .map_or(1.0, |weight| 1.0 / weight.max(f64::EPSILON))
}

fn require_entity(idx: &GraphIndex, entity: EntityId) -> Result<(), GraphError> {
    if idx.entities.contains(&entity) {
        Ok(())
    } else {
        Err(GraphError::NotFound)
    }
}

fn increment_degree(degrees: &mut BTreeMap<EntityId, usize>, entity: EntityId) {
    if let Some(degree) = degrees.get_mut(&entity) {
        *degree = degree.saturating_add(1);
    }
}

fn is_better(current: Option<&(f64, usize)>, cost: f64, hops: usize) -> bool {
    current.is_none_or(|(best_cost, best_hops)| {
        cost.total_cmp(best_cost) == Ordering::Less
            || (cost.total_cmp(best_cost) == Ordering::Equal && hops < *best_hops)
    })
}

fn stale_state(best: &BTreeMap<EntityId, (f64, usize)>, state: QueueState) -> bool {
    best.get(&state.node).is_some_and(|(cost, hops)| {
        *hops != state.hops || cost.total_cmp(&state.cost) != Ordering::Equal
    })
}

#[derive(Debug, Clone, Copy)]
struct QueueState {
    cost: f64,
    hops: usize,
    node: EntityId,
}

impl QueueState {
    fn new(cost: f64, hops: usize, node: EntityId) -> Self {
        Self { cost, hops, node }
    }
}

impl PartialEq for QueueState {
    fn eq(&self, other: &Self) -> bool {
        self.cost.total_cmp(&other.cost) == Ordering::Equal
            && self.hops == other.hops
            && self.node == other.node
    }
}

impl Eq for QueueState {}

impl PartialOrd for QueueState {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for QueueState {
    fn cmp(&self, other: &Self) -> Ordering {
        self.cost
            .total_cmp(&other.cost)
            .then_with(|| self.hops.cmp(&other.hops))
            .then_with(|| self.node.cmp(&other.node))
    }
}

#[derive(Debug)]
struct OrderedHit {
    position: usize,
    entity: EntityId,
    attr: AttrId,
    hit: SearchHit,
}

fn search_hit_order(a: &OrderedHit, b: &OrderedHit) -> Ordering {
    (a.position, a.entity, &a.attr).cmp(&(b.position, b.entity, &b.attr))
}
