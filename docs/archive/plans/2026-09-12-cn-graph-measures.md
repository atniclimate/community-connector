# cn-graph Event Measures + cn-api JSON Call Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to
> implement this plan task-by-task in this session (single build session per
> SESSION_ROSTER.yaml S-R4a: `mode: plan-then-code`, `run_as: main`). Do NOT
> spawn a fresh subagent per task — S-R4a's own delegation budget is exactly
> one `claim-verifier` subagent, dispatched only after all tasks below are
> green, to independently recompute betweenness for three named nodes by hand.

**Goal:** Add five deterministic graph measures (degree, single-tie,
shared-committee Jaccard, betweenness, eccentricity) to `cn-graph`, each with a
plain-language explanation string, and expose them through one new `cn-api`
JSON call (`graph_measures`, in the style of `query_paths`) plus a matching
`cn-wasm` export.

**Architecture:** All five measures are pure functions over `cn_graph::GraphIndex`
(built from a `cn_perm::Projection` — the only read path, I2). No new crates;
no widening of the projection. `cn-api::graph_measures` composes them into one
JSON envelope the same way `query_paths`/`query_neighborhood` already do
(`parse_json` request DTO -> `index_for_request` -> `respond`).

**Tech Stack:** Rust workspace (`core/`), serde/serde_json, existing
`BTreeMap`-adjacency `GraphIndex`. No petgraph, no Louvain (out of scope for
this session per discovery memo Track B ranked shortlist).

**Spec:** Session prompt for SESSION_ROSTER.yaml `S-R4a` (2026-09-12 pickup);
`docs/research/discovery-2026-09-12.md` Track B table (lines 84-139).

## Global Constraints

- cn-perm remains the only place permission logic lives (I2) — every measure
  takes the projection/index `cn-perm` already filtered and never re-derives
  visibility.
- Deterministic output: iterate `BTreeMap`/`BTreeSet` in key order; no
  `HashMap`; fixed tie-breaks; two runs on the same projection must produce
  byte-identical JSON.
- Every measure returns a value AND a plain-language `explanation: String`.
- Scope fence: `core/crates/cn-graph`, `core/crates/cn-api`,
  `core/crates/cn-wasm` (export only), their tests, `SESSION_ROSTER.yaml`. No
  `app/` code. Do not push.
- No new crates/dependencies.
- Betweenness: Brandes' algorithm, exact, unweighted, normalized by
  `(n-1)(n-2)`. Eccentricity: BFS from each node; unreachable nodes report
  their own component separately (never silently merged with the largest
  component).
- Gate: `cargo test --workspace` (from `core/`) green, plus
  `pwsh scripts/check-all.ps1` 12/12 (11/12 is the pre-existing S-R0
  toolchain-drift clippy failure at `core/cli/src/intake/keymat.rs:258` — not
  this session's to fix; note it in the completion report rather than
  silently accepting it if it's still 11/12).

---

## File Structure

- `core/crates/cn-graph/src/query.rs` — add four new public measure functions
  (`degree_measures`, `shared_committee_jaccard`, `betweenness`,
  `eccentricity`) and their result structs, plus small private helpers
  (`all_neighbors`, `committees_of`, `connected_components`). Existing
  `degrees()` is kept as-is and reused by `degree_measures`.
- `core/crates/cn-graph/tests/blueprint.rs` — add tests for all four new
  functions, including the planted-bridge-node betweenness assertion.
- `core/crates/cn-api/src/dto.rs` — add `GraphMeasuresRequest`,
  `JaccardPairRequest`, `GraphMeasures` (response) DTOs.
- `core/crates/cn-api/src/lib.rs` — add `pub fn graph_measures(...)` +
  `fn graph_measures_impl(...)`, wired the same way as `query_paths`.
- `core/crates/cn-api/tests/graph_measures.rs` (new file) — integration test
  loading the real ATNI convention fixture
  (`fixtures/templates/atni-convention.template.json` +
  `fixtures/groups/atni-convention.ops.jsonl`) through
  `load_group_begin`/`load_ops_chunk`/`load_group_commit`, then calling
  `graph_measures` and asserting structural correctness.
- `core/crates/cn-wasm/src/lib.rs` — add a `graph_measures` wasm-bindgen
  method mirroring `query_paths`, inside the existing `#[cfg(target_arch =
  "wasm32")] mod bindings` block. Export only — no new logic.
- `SESSION_ROSTER.yaml` — flip S-R4a's `status`/`outcome` once the gate is
  green.

## Interfaces (fixed now so later tasks don't drift)

```rust
// cn-graph/src/query.rs

pub struct DegreeMeasure {
    pub entity: EntityId,
    pub degree: usize,
    pub single_tie: bool,
    pub explanation: String,
}
pub fn degree_measures(idx: &GraphIndex) -> BTreeMap<EntityId, DegreeMeasure>;

pub struct JaccardMeasure {
    pub a: EntityId,
    pub b: EntityId,
    pub shared: usize,
    pub union: usize,
    pub value: f64,
    pub explanation: String,
}
pub fn shared_committee_jaccard(
    idx: &GraphIndex,
    a: EntityId,
    b: EntityId,
    membership_kind: &KindId,
) -> Result<JaccardMeasure, GraphError>;

pub struct BetweennessMeasure {
    pub entity: EntityId,
    pub value: f64,
    pub explanation: String,
}
pub fn betweenness(idx: &GraphIndex) -> BTreeMap<EntityId, BetweennessMeasure>;

pub struct EccentricityMeasure {
    pub entity: EntityId,
    pub steps: usize,
    pub component: EntityId, // min EntityId in the node's connected component
    pub explanation: String,
}
pub fn eccentricity(idx: &GraphIndex) -> BTreeMap<EntityId, EccentricityMeasure>;
```

```rust
// cn-api/src/dto.rs
pub(crate) struct JaccardPairRequest { pub(crate) a: EntityId, pub(crate) b: EntityId }
pub(crate) struct GraphMeasuresRequest {
    pub(crate) membership_kind: KindId,
    pub(crate) jaccard_pairs: Vec<JaccardPairRequest>, // #[serde(default)]
}
pub(crate) struct GraphMeasures {
    pub(crate) degree: BTreeMap<EntityId, cn_graph::DegreeMeasure>,
    pub(crate) betweenness: BTreeMap<EntityId, cn_graph::BetweennessMeasure>,
    pub(crate) eccentricity: BTreeMap<EntityId, cn_graph::EccentricityMeasure>,
    pub(crate) shared_committee_jaccard: Vec<cn_graph::JaccardMeasure>,
}
```

`Api::graph_measures(&mut self, group_id: &str, viewer_ctx_json: &str, request_json: &str) -> String`.

Betweenness normalization: run Brandes accumulating `centrality[v] += delta[v]`
for every source `s` (no post-hoc halving — this already sums over ordered
pairs, matching the "sum over s,t of sigma(s,t|v)/sigma(s,t)" formula given in
the brief), then `value = centrality[v] / ((n-1)*(n-2))` when `n > 2`, else
`0.0`. Explanation: `format!("sits on {:.1}% of shortest paths between other
people", value * 100.0)`.

`shared_committee_jaccard` reads committee membership from the index's own
adjacency: for entity `e`, its committees are every adjacency entry from `e`
with `forward == true` and `kind == membership_kind` (this is exactly the
`member_of` person->committee direction, whichever the caller names).

`eccentricity`'s `component` field comes from one BFS-based component pass
(union by "smallest EntityId in the component"), computed once and reused for
every node's `EccentricityMeasure`, so two nodes in the same connected
component always report the same `component` value — this is how "unreachable
nodes report their component separately" surfaces in the output.

---

## Task 1: `degree_measures` in cn-graph

**Files:**
- Modify: `core/crates/cn-graph/src/query.rs`
- Test: `core/crates/cn-graph/tests/blueprint.rs`

**Interfaces:**
- Produces: `DegreeMeasure`, `degree_measures(idx) -> BTreeMap<EntityId, DegreeMeasure>`.

- [ ] **Step 1: Write the failing test** (append to `blueprint.rs`)

```rust
#[test]
fn degree_measures_flag_single_tie_and_explain() {
    let p = projection(
        vec![entity(1, "person"), entity(2, "person"), entity(3, "person")],
        vec![
            edge(1, 1, 2, "connected_to", false, None),
            edge(2, 1, 3, "connected_to", false, None),
        ],
    );
    let idx = GraphIndex::build(&p);
    let measures = degree_measures(&idx);
    assert_eq!(measures[&entity_id(1)].degree, 2);
    assert!(!measures[&entity_id(1)].single_tie);
    assert_eq!(measures[&entity_id(2)].degree, 1);
    assert!(measures[&entity_id(2)].single_tie);
    assert!(measures[&entity_id(2)].explanation.contains('1'));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run (from `core/`): `cargo test -p cn-graph degree_measures_flag_single_tie_and_explain`
Expected: FAIL with "cannot find function `degree_measures`" / "cannot find struct `DegreeMeasure`".

- [ ] **Step 3: Write minimal implementation** (in `query.rs`, near `degrees`)

```rust
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
                "connected to only one other person".to_string()
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
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p cn-graph degree_measures_flag_single_tie_and_explain`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add core/crates/cn-graph/src/query.rs core/crates/cn-graph/tests/blueprint.rs
git commit -m "feat(cn-graph): add degree_measures with single-tie flag and explanation"
```

---

## Task 2: `shared_committee_jaccard` in cn-graph

**Files:**
- Modify: `core/crates/cn-graph/src/query.rs`
- Test: `core/crates/cn-graph/tests/blueprint.rs`

**Interfaces:**
- Consumes: `GraphIndex` (private `adjacency`/`entities` fields, same module).
- Produces: `JaccardMeasure`, `shared_committee_jaccard(idx, a, b, membership_kind) -> Result<JaccardMeasure, GraphError>`.

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn shared_committee_jaccard_counts_overlap_and_rejects_missing() {
    let p = projection(
        vec![
            entity(1, "person"),
            entity(2, "person"),
            entity(3, "person"),
            entity(10, "committee"),
            entity(11, "committee"),
            entity(12, "committee"),
        ],
        vec![
            edge(1, 1, 10, "member_of", true, None),
            edge(2, 1, 11, "member_of", true, None),
            edge(3, 2, 11, "member_of", true, None),
            edge(4, 2, 12, "member_of", true, None),
        ],
    );
    let idx = GraphIndex::build(&p);
    let member_of = kind("member_of");
    let overlap = shared_committee_jaccard(&idx, entity_id(1), entity_id(2), &member_of)
        .expect("query");
    assert_eq!(overlap.shared, 1);
    assert_eq!(overlap.union, 3);
    assert!((overlap.value - 1.0 / 3.0).abs() < 1e-9);
    assert!(overlap.explanation.contains('1'));

    let none = shared_committee_jaccard(&idx, entity_id(1), entity_id(3), &member_of)
        .expect("query");
    assert_eq!(none.shared, 0);
    assert_eq!(none.union, 2);
    assert_eq!(none.value, 0.0);

    assert_eq!(
        shared_committee_jaccard(&idx, entity_id(1), entity_id(99), &member_of),
        Err(GraphError::NotFound)
    );
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p cn-graph shared_committee_jaccard_counts_overlap_and_rejects_missing`
Expected: FAIL (function/struct not found).

- [ ] **Step 3: Write minimal implementation**

```rust
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

fn committees_of(idx: &GraphIndex, entity: EntityId, membership_kind: &KindId) -> BTreeSet<EntityId> {
    idx.adjacency
        .get(&entity)
        .into_iter()
        .flatten()
        .filter(|adj| adj.forward && &adj.kind == membership_kind)
        .map(|adj| adj.to)
        .collect()
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p cn-graph shared_committee_jaccard_counts_overlap_and_rejects_missing`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add core/crates/cn-graph/src/query.rs core/crates/cn-graph/tests/blueprint.rs
git commit -m "feat(cn-graph): add shared_committee_jaccard over member_of edges"
```

---

## Task 3: `betweenness` (Brandes') in cn-graph

**Files:**
- Modify: `core/crates/cn-graph/src/query.rs`
- Test: `core/crates/cn-graph/tests/blueprint.rs`

**Interfaces:**
- Produces: `BetweennessMeasure`, `betweenness(idx) -> BTreeMap<EntityId, BetweennessMeasure>`.

- [ ] **Step 1: Write the failing test** (planted bridge node)

```rust
#[test]
fn betweenness_ranks_the_planted_bridge_above_every_non_bridge_node() {
    // Two triangles joined only through entity 4 (the bridge).
    let p = projection(
        vec![
            entity(1, "person"),
            entity(2, "person"),
            entity(3, "person"),
            entity(4, "person"),
            entity(5, "person"),
            entity(6, "person"),
            entity(7, "person"),
        ],
        vec![
            edge(1, 1, 2, "connected_to", false, None),
            edge(2, 2, 3, "connected_to", false, None),
            edge(3, 3, 1, "connected_to", false, None),
            edge(4, 3, 4, "connected_to", false, None),
            edge(5, 4, 5, "connected_to", false, None),
            edge(6, 5, 6, "connected_to", false, None),
            edge(7, 6, 7, "connected_to", false, None),
            edge(8, 7, 5, "connected_to", false, None),
        ],
    );
    let idx = GraphIndex::build(&p);
    let scores = betweenness(&idx);
    let bridge = scores[&entity_id(4)].value;
    for (entity, measure) in &scores {
        if *entity != entity_id(4) {
            assert!(
                measure.value < bridge,
                "{entity:?} scored {} >= bridge's {bridge}",
                measure.value
            );
        }
    }
    assert!(scores[&entity_id(4)].explanation.contains('%'));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p cn-graph betweenness_ranks_the_planted_bridge_above_every_non_bridge_node`
Expected: FAIL (function/struct not found).

- [ ] **Step 3: Write minimal implementation**

```rust
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
    let n = nodes.len();
    let mut centrality: BTreeMap<EntityId, f64> = nodes.iter().map(|&v| (v, 0.0)).collect();

    for &s in &nodes {
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

    let normalization = if n > 2 { ((n - 1) * (n - 2)) as f64 } else { 0.0 };
    nodes
        .into_iter()
        .map(|entity| {
            let raw = centrality[&entity];
            let value = if normalization > 0.0 { raw / normalization } else { 0.0 };
            let explanation = format!(
                "sits on {:.1}% of shortest paths between other people",
                value * 100.0
            );
            (entity, BetweennessMeasure { entity, value, explanation })
        })
        .collect()
}

/// All adjacency entries for a node, ignoring `forward`/direction — used by
/// measures defined as undirected (betweenness, eccentricity).
fn all_neighbors(idx: &GraphIndex, node: EntityId) -> impl Iterator<Item = &Adjacency> {
    idx.adjacency.get(&node).into_iter().flatten()
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p cn-graph betweenness_ranks_the_planted_bridge_above_every_non_bridge_node`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add core/crates/cn-graph/src/query.rs core/crates/cn-graph/tests/blueprint.rs
git commit -m "feat(cn-graph): add exact Brandes' betweenness centrality"
```

---

## Task 4: `eccentricity` in cn-graph

**Files:**
- Modify: `core/crates/cn-graph/src/query.rs`
- Test: `core/crates/cn-graph/tests/blueprint.rs`

**Interfaces:**
- Consumes: `all_neighbors` (Task 3).
- Produces: `EccentricityMeasure`, `eccentricity(idx) -> BTreeMap<EntityId, EccentricityMeasure>`.

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn eccentricity_reports_farthest_steps_and_separates_components() {
    // A 4-node line (1-2-3-4) plus a disconnected pair (5-6).
    let p = projection(
        vec![
            entity(1, "person"),
            entity(2, "person"),
            entity(3, "person"),
            entity(4, "person"),
            entity(5, "person"),
            entity(6, "person"),
        ],
        vec![
            edge(1, 1, 2, "connected_to", false, None),
            edge(2, 2, 3, "connected_to", false, None),
            edge(3, 3, 4, "connected_to", false, None),
            edge(4, 5, 6, "connected_to", false, None),
        ],
    );
    let idx = GraphIndex::build(&p);
    let ecc = eccentricity(&idx);
    assert_eq!(ecc[&entity_id(1)].steps, 3);
    assert_eq!(ecc[&entity_id(2)].steps, 2);
    assert_eq!(ecc[&entity_id(1)].component, ecc[&entity_id(4)].component);
    assert_ne!(ecc[&entity_id(1)].component, ecc[&entity_id(5)].component);
    assert_eq!(ecc[&entity_id(5)].steps, 1);
    assert!(ecc[&entity_id(1)].explanation.contains('3'));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p cn-graph eccentricity_reports_farthest_steps_and_separates_components`
Expected: FAIL (function/struct not found).

- [ ] **Step 3: Write minimal implementation**

```rust
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
/// they are mutually reachable — this is how disconnected nodes report their
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
                    if !dist.contains_key(&adj.to) {
                        dist.insert(adj.to, d + 1);
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
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p cn-graph eccentricity_reports_farthest_steps_and_separates_components`
Expected: PASS.

- [ ] **Step 5: Run the whole cn-graph test suite**

Run: `cargo test -p cn-graph`
Expected: all PASS (existing + 4 new tests).

- [ ] **Step 6: Commit**

```bash
git add core/crates/cn-graph/src/query.rs core/crates/cn-graph/tests/blueprint.rs
git commit -m "feat(cn-graph): add BFS eccentricity with per-component grouping"
```

---

## Task 5: `cn-api` DTOs + `graph_measures` call

**Files:**
- Modify: `core/crates/cn-api/src/dto.rs`
- Modify: `core/crates/cn-api/src/lib.rs`
- Test: `core/crates/cn-api/tests/graph_measures.rs` (new)

**Interfaces:**
- Consumes: `cn_graph::{degree_measures, shared_committee_jaccard, betweenness, eccentricity}` (Tasks 1-4); `Api::index_for_request` (existing, `lib.rs:503`); `wire::{parse_json, respond}`; `error::{ApiError}`; `lib::graph_error` (existing, maps `GraphError::NotFound` -> `ApiError::not_found()`).
- Produces: `Api::graph_measures(&mut self, group_id: &str, viewer_ctx_json: &str, request_json: &str) -> String`.

- [ ] **Step 1: Write the failing test** (new file `core/crates/cn-api/tests/graph_measures.rs`)

```rust
//! Integration coverage for S-R4a: cn-api's new graph_measures JSON call,
//! exercised against the real, fully synthetic ATNI convention fixture
//! (D-094c) rather than an inline projection, per this session's own gate.

use cn_api::Api;
use serde_json::{Value, json};

const ATNI_TEMPLATE: &str =
    include_str!("../../../../fixtures/templates/atni-convention.template.json");
const ATNI_OPS: &str = include_str!("../../../../fixtures/groups/atni-convention.ops.jsonl");

const GROUP_ID: &str = "00000000-0000-0000-0000-0000000dbba0";
// A "governance" member from the fixture (active MembershipAdd), so the
// projection is non-empty (Group-visibility values require Group access).
const GOVERNANCE_VIEWER: &str = "00000000-0000-0000-0000-0000000de2bd";

fn ok(json: &str) -> Value {
    let value: Value = serde_json::from_str(json).expect("valid envelope json");
    assert!(value.get("ok").is_some() ^ value.get("err").is_some());
    value.get("ok").cloned().unwrap_or_else(|| panic!("ok envelope: {json}"))
}

fn viewer() -> String {
    json!({ "kind": "person", "person": GOVERNANCE_VIEWER }).to_string()
}

fn load_atni_api() -> Api {
    let mut api = Api::new();
    ok(&api.load_group_begin(GROUP_ID, &viewer(), ATNI_TEMPLATE));
    ok(&api.load_ops_chunk(GROUP_ID, ATNI_OPS));
    ok(&api.load_group_commit(GROUP_ID, 1_000));
    api
}

#[test]
fn graph_measures_returns_all_five_measures_over_the_atni_fixture() {
    let mut api = load_atni_api();
    let projection: Value = ok(&api.projection(GROUP_ID, &viewer()));
    let entities = projection["entities"]
        .as_array()
        .expect("entities array");
    assert!(entities.len() > 2, "fixture projection unexpectedly empty");
    let person_ids: Vec<&str> = entities
        .iter()
        .filter(|e| e["kind"] == "person")
        .map(|e| e["id"].as_str().expect("id"))
        .take(2)
        .collect();
    assert_eq!(person_ids.len(), 2, "fixture must contain at least two people");

    let request = json!({
        "membership_kind": "member_of",
        "jaccard_pairs": [{ "a": person_ids[0], "b": person_ids[1] }],
    })
    .to_string();
    let measures = ok(&api.graph_measures(GROUP_ID, &viewer(), &request));

    let degree = measures["degree"].as_object().expect("degree map");
    assert_eq!(degree.len(), entities.len());
    for (_, measure) in degree {
        assert!(measure["explanation"].as_str().unwrap_or_default().len() > 0);
    }

    let betweenness = measures["betweenness"].as_object().expect("betweenness map");
    assert_eq!(betweenness.len(), entities.len());
    assert!(
        betweenness
            .values()
            .any(|m| m["value"].as_f64().unwrap_or(0.0) > 0.0),
        "at least one entity should sit on some shortest path in this fixture"
    );

    let eccentricity = measures["eccentricity"].as_object().expect("eccentricity map");
    assert_eq!(eccentricity.len(), entities.len());

    let jaccard = measures["shared_committee_jaccard"]
        .as_array()
        .expect("jaccard array");
    assert_eq!(jaccard.len(), 1);
    let pair = &jaccard[0];
    let shared = pair["shared"].as_u64().expect("shared");
    let union = pair["union"].as_u64().expect("union");
    assert!(shared <= union);
    if union > 0 {
        let expected = shared as f64 / union as f64;
        assert!((pair["value"].as_f64().unwrap() - expected).abs() < 1e-9);
    } else {
        assert_eq!(pair["value"].as_f64().unwrap(), 0.0);
    }
}

#[test]
fn graph_measures_rejects_unknown_jaccard_entity() {
    let mut api = load_atni_api();
    let request = json!({
        "membership_kind": "member_of",
        "jaccard_pairs": [{
            "a": "00000000-0000-0000-0000-000000000000",
            "b": "00000000-0000-0000-0000-000000000000",
        }],
    })
    .to_string();
    let response: Value =
        serde_json::from_str(&api.graph_measures(GROUP_ID, &viewer(), &request))
            .expect("valid envelope json");
    assert_eq!(response["err"]["code"], "not_found");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run (from `core/`): `cargo test -p cn-api --test graph_measures`
Expected: FAIL to compile — `graph_measures` method does not exist on `Api`.

- [ ] **Step 3: Write minimal implementation**

In `core/crates/cn-api/src/dto.rs`, add near `PathRequest`/`NeighborhoodRequest`:

```rust
#[derive(Debug, Deserialize)]
pub(crate) struct JaccardPairRequest {
    pub(crate) a: EntityId,
    pub(crate) b: EntityId,
}

#[derive(Debug, Deserialize)]
pub(crate) struct GraphMeasuresRequest {
    pub(crate) membership_kind: KindId,
    #[serde(default)]
    pub(crate) jaccard_pairs: Vec<JaccardPairRequest>,
}

#[derive(Debug, Serialize)]
pub(crate) struct GraphMeasures {
    pub(crate) degree: BTreeMap<EntityId, cn_graph::DegreeMeasure>,
    pub(crate) betweenness: BTreeMap<EntityId, cn_graph::BetweennessMeasure>,
    pub(crate) eccentricity: BTreeMap<EntityId, cn_graph::EccentricityMeasure>,
    pub(crate) shared_committee_jaccard: Vec<cn_graph::JaccardMeasure>,
}
```

Add `use cn_graph;` alongside the existing `use cn_graph::PathConstraints;` import
(change it to `use cn_graph::{self, PathConstraints};`).

In `core/crates/cn-api/src/lib.rs`:

1. Add to the `dto::{...}` import list: `GraphMeasures, GraphMeasuresRequest,`.
2. Add the public method next to `query_neighborhood`:

```rust
    /// Computes degree/single-tie/betweenness/eccentricity for every entity
    /// plus shared-committee Jaccard for the requested pairs, all over the
    /// viewer projection (ADR-003 D1; discovery-2026-09-12.md Track B).
    pub fn graph_measures(
        &mut self,
        group_id: &str,
        viewer_ctx_json: &str,
        request_json: &str,
    ) -> String {
        respond(|| self.graph_measures_impl(group_id, viewer_ctx_json, request_json))
    }
```

3. Add the private impl next to `query_neighborhood_impl`:

```rust
    fn graph_measures_impl(
        &mut self,
        group_id: &str,
        viewer_ctx_json: &str,
        request_json: &str,
    ) -> Result<GraphMeasures, ApiError> {
        let request: GraphMeasuresRequest = parse_json(request_json)?;
        let (_projection, index) = self.index_for_request(group_id, viewer_ctx_json)?;
        let degree = cn_graph::degree_measures(&index);
        let betweenness = cn_graph::betweenness(&index);
        let eccentricity = cn_graph::eccentricity(&index);
        let mut shared_committee_jaccard = Vec::with_capacity(request.jaccard_pairs.len());
        for pair in &request.jaccard_pairs {
            shared_committee_jaccard.push(
                cn_graph::shared_committee_jaccard(
                    &index,
                    pair.a,
                    pair.b,
                    &request.membership_kind,
                )
                .map_err(graph_error)?,
            );
        }
        Ok(GraphMeasures {
            degree,
            betweenness,
            eccentricity,
            shared_committee_jaccard,
        })
    }
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p cn-api --test graph_measures`
Expected: PASS (both tests).

- [ ] **Step 5: Run the full cn-api suite**

Run: `cargo test -p cn-api`
Expected: all PASS (existing + 2 new).

- [ ] **Step 6: Commit**

```bash
git add core/crates/cn-api/src/dto.rs core/crates/cn-api/src/lib.rs core/crates/cn-api/tests/graph_measures.rs
git commit -m "feat(cn-api): expose graph_measures as one JSON call over cn-graph"
```

---

## Task 6: `cn-wasm` export

**Files:**
- Modify: `core/crates/cn-wasm/src/lib.rs`

**Interfaces:**
- Consumes: `cn_api::Api::graph_measures` (Task 5).
- Produces: `CnApi::graph_measures` wasm-bindgen method (native code is `#[cfg(target_arch = "wasm32")]`-gated, so no native test exercises it directly — verified by `cargo build --target wasm32-unknown-unknown -p cn-wasm` if that target is installed, otherwise by `cargo check -p cn-wasm` on native, which still type-checks the non-wasm re-export path).

- [ ] **Step 1: Add the binding** (inside `mod bindings`, next to `query_neighborhood`)

```rust
        pub fn graph_measures(
            &mut self,
            group_id: &str,
            viewer_ctx_json: &str,
            request_json: &str,
        ) -> String {
            self.inner
                .graph_measures(group_id, viewer_ctx_json, request_json)
        }
```

- [ ] **Step 2: Verify it compiles**

Run: `cargo check -p cn-wasm`
Expected: PASS (the `#[cfg(target_arch = "wasm32")]` block is not compiled
natively, so this mainly checks the crate still builds; if a wasm32 target is
installed, also run `cargo check --target wasm32-unknown-unknown -p cn-wasm`).

- [ ] **Step 3: Commit**

```bash
git add core/crates/cn-wasm/src/lib.rs
git commit -m "feat(cn-wasm): export graph_measures binding"
```

---

## Task 7: Workspace gate, SESSION_ROSTER.yaml update, claim-verifier

**Files:**
- Run: `cargo test --workspace` (from `core/`)
- Run: `pwsh scripts/check-all.ps1`
- Modify: `SESSION_ROSTER.yaml` (S-R4a entry: `status`, `outcome`)

- [ ] **Step 1: Full workspace test gate**

Run (from `core/`): `cargo test --workspace 2>&1 | Tee-Object _private/scratch/r4a-cargo.log | Select-String "test result:|FAILED|panicked"`
Expected: every crate's `test result: ok`, no FAILED/panicked lines.

- [ ] **Step 2: check-all gate**

Run: `pwsh scripts/check-all.ps1`
Expected: 12/12, OR 11/12 with the sole failure being the pre-existing S-R0
rust-clippy toolchain-drift lint at `core/cli/src/intake/keymat.rs:258` (not
touched by this session) — paste the actual summary either way.

- [ ] **Step 3: Dispatch the one claim-verifier subagent**

Per this session's own delegation budget (SESSION_ROSTER.yaml S-R4a
`subagents`), dispatch exactly one `claim-verifier` (claude-sonnet-5, medium
effort) with: the three node ids used in Task 3's planted-bridge fixture (the
bridge, `entity_id(4)`, plus two non-bridge nodes, e.g. `entity_id(1)` and
`entity_id(6)`), the fixture's edge list, and Brandes' formula from this plan.
It must recompute betweenness for those three nodes BY HAND from the edges
(not by reading `query.rs`) and confirm the ranking (and ideally the
normalized values) match what `betweenness()` returns. Do not mark S-R4a done
until this agrees.

- [ ] **Step 4: Update SESSION_ROSTER.yaml**

Set S-R4a's `status: done` and write an `outcome:` string covering: files
changed, the five measures shipped, the two gate results (paste actual
numbers), and the claim-verifier's verdict — following the same style as
S-R2's existing `outcome:` entry.

- [ ] **Step 5: Commit**

```bash
git add SESSION_ROSTER.yaml
git commit -m "docs(planning): record S-R4a outcome (cn-graph event measures)"
```

---

## Self-Review Notes

- **Spec coverage:** degree (Task 1, "exists; expose it" — reuses `degrees()`),
  single_tie (Task 1), shared_committee_jaccard (Task 2), betweenness (Task 3),
  eccentricity (Task 4) — all five measures from the brief. cn-api exposure as
  one call — Task 5. cn-wasm export-only — Task 6. Fixture requirement (ATNI
  if landed) — Task 5's integration test uses the real ATNI fixture; Tasks
  1-4's unit tests use small inline projections (existing `blueprint.rs`
  style) because they need precisely engineered graph shapes (a planted
  bridge, a disconnected pair) that the pseudo-randomly generated ATNI fixture
  does not guarantee — the ATNI fixture coverage lives at the cn-api
  integration level instead, which is where "on the fixture" matters most
  (real permission projection, real entity/edge kinds). One claim-verifier —
  Task 7. No new crates — confirmed (no Cargo.toml edits anywhere in this
  plan). Scope fence respected — no `app/` files touched.
- **Type consistency:** `EntityId`, `KindId`, `GraphError` all reused from
  existing `cn-model`/`cn-graph` imports already present in `query.rs`;
  `BTreeSet` is already imported at the top of `query.rs` (`use
  std::collections::{BTreeMap, BTreeSet, BinaryHeap, VecDeque};`) so Task 2/4's
  helpers need no new imports.
