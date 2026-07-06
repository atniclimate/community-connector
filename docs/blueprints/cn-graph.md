# Blueprint: cn-graph (Phase 2, authored by director)

Graph queries over PROJECTIONS ONLY (ADR-001 A-B1 / ADR-003 D1): every public
function takes `&cn_perm::Projection`; there is NO API accepting GroupState
or raw ops. Because hidden objects are absent from projections, query results
structurally cannot leak them. Wasm-safe throughout.

## Dependencies

```toml
[dependencies]
cn-model = { path = "../cn-model" }
cn-perm = { path = "../cn-perm" }
serde = { version = "1", features = ["derive"] }
thiserror = "2"
```

## API (module: query)

```rust
/// Adjacency index built once per projection revision; all queries take it.
pub struct GraphIndex { /* private adjacency from Projection */ }
impl GraphIndex { pub fn build(p: &cn_perm::Projection) -> Self; }

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PathConstraints {
    pub allowed_edge_kinds: Option<Vec<KindId>>,   // None = all
    pub max_hops: Option<usize>,                   // None = unbounded
    pub weighted: bool,       // true: Dijkstra over weight (missing weight = 1.0);
                              // false: BFS hop count
    pub respect_direction: bool,   // false: treat all edges as undirected
}

pub fn shortest_path(idx: &GraphIndex, from: EntityId, to: EntityId,
    c: &PathConstraints) -> Result<Option<Path>, GraphError>;
// Ok(None) = no route. UNKNOWN ids -> GraphError::NotFound (ADR-003 A-B4:
// hidden and missing are the same error).

pub struct Path { pub nodes: Vec<EntityId>, pub edges: Vec<EdgeId>, pub cost: f64 }

pub fn neighborhood(idx: &GraphIndex, center: EntityId, hops: usize,
    c: &PathConstraints) -> Result<Neighborhood, GraphError>;
pub struct Neighborhood { pub center: EntityId, pub layers: Vec<Vec<EntityId>> }
// layers[0] = 1-hop, deduped, no re-visits; stable ordering (sort ids).

pub fn degrees(idx: &GraphIndex) -> BTreeMap<EntityId, usize>;

pub fn search(p: &cn_perm::Projection, q: &SearchQuery) -> Vec<SearchHit>;
pub struct SearchQuery { pub text: String,             // case-insensitive substring
    pub kinds: Option<Vec<KindId>>, pub limit: usize }
pub struct SearchHit { pub entity: EntityId, pub kind: KindId,
    pub matched_attr: AttrId, pub snippet: String }
// Matches over projected Text/Enum/Tags/Link attribute values only; snippet
// is the matched value truncated to 80 chars on a char boundary. Stable
// order: (match position, entity id). Empty/whitespace query -> empty vec.

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum GraphError { #[error("not found")] NotFound }
```

## Test obligations

1. Need-to-solution routing (fisheries fixture shapes): need -> need_met_by
   -> skill/person paths found; allowed_edge_kinds constraint excludes
   other routes; weighted=true prefers the higher-weight... note: Dijkstra
   MINIMIZES cost - define edge cost = 1.0 / weight.max(f64::EPSILON) when
   weighted (stronger tie = cheaper) and TEST it.
2. respect_direction on and off change reachability accordingly.
3. max_hops cuts long routes; hop-0 neighborhood is empty layers.
4. NotFound for absent ids (which includes hidden - construct a projection
   lacking an entity and assert the error equals the truly-missing case).
5. Determinism: identical inputs give identical outputs including ordering;
   neighborhood layers deduped across layers.
6. search: case-insensitivity, kind filter, limit, tags matching, stable
   ordering, empty query.

Verification: fmt, clippy -D warnings, test --workspace,
`cargo build --target wasm32-unknown-unknown -p cn-graph`.
