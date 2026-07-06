# ADR-003: WASM Boundary Shape

- Status: draft - amended after adversarial round 1 (REDESIGN verdict);
  awaiting round 2 (final)
- Date: 2026-07-06
- Phase: 1
- Drivers: architecture stances (Rust core is the single source of truth; the
  frontend renders projections it is handed; replacing the renderer never
  touches the core), R6 (queries fast at low-thousands of nodes), I2 (no
  permission logic outside cn-perm), I3 (typed errors), I7 (versioned
  formats)

## Context

cn-wasm is the only door between the TypeScript app and the Rust core. The
same core crates power the native `cn` CLI without wasm. The boundary must be
small enough to keep the core replaceable-renderer-proof, and honest enough
that no unfiltered data can cross it (ADR-001 A-B1: cn-graph and the app see
projections only).

## Decision

### D1. Boundary surface (v0)

All calls are namespaced by group where applicable. The complete surface:

```
core_info() -> { core_version, boundary_version, supported_schema_majors }
load_group(group_id, template_json, ops_jsonl) -> Result<LoadReport>
submit_ops(group_id, ops_json) -> Result<SubmitReport>       // cn-perm authorizes (ADR-002 A-B6)
projection(group_id, viewer_ctx_json) -> Result<ProjectionJson>
query_paths(group_id, viewer_ctx_json, path_request_json) -> Result<PathsJson>
query_neighborhood(group_id, viewer_ctx_json, request_json) -> Result<NeighborhoodJson>
search(group_id, viewer_ctx_json, query_json) -> Result<SearchJson>
validation_report(group_id) -> Result<ReportJson>
export_snapshot(group_id, viewer_ctx_json, options_json) -> Result<SnapshotJson>
```

Every query takes the viewer context; the core computes (and caches) the
projection internally. There is NO call that returns raw state, raw ops, or a
projection for a viewer the caller did not name. The app cannot ask a
question the permission engine has not already filtered.

### D2. JSON strings at the boundary, versioned

v0 crosses the boundary as UTF-8 JSON strings with explicit
`boundary_version` (I7): simple, debuggable, schema-checkable, and identical
to the CLI's I/O formats, so one serde model serves both frontends. Declared
escape hatch: if Phase 3 profiling shows projection transfer dominating,
projection GEOMETRY-ADJACENT data (id-indexed arrays the renderer iterates)
may move to typed-array views in a boundary_version bump - a performance
change confined to cn-wasm and app/src/wasm, invisible to the core crates.

### D3. Projection identity and invalidation

A `ProjectionJson` carries `{ group_id, viewer_fingerprint, revision }`.
`revision` increments on every accepted submit; the app treats a projection
as stale when revisions move and re-requests. The core caches the last
projection per (group, viewer_fingerprint) so re-requests after unrelated
UI activity are cheap. No incremental diffs in v0 (renderer redraw at 5k
nodes is cheaper than a diff protocol; revisit only with profiling evidence).

### D4. Errors are typed values, not strings

Every fallible call returns a discriminated result:
`{ ok: ... } | { err: { code, message, details } }` with `code` from a closed
enum shared with the CLI (schema-versioned). wasm-bindgen maps `err` to a
thrown JS error carrying the payload. Nothing is stringly-typed, nothing is
swallowed (I3); quarantine and validation findings arrive in reports, not
exceptions.

### D5. Memory and threading

Single-threaded wasm (no atomics/shared memory in v0); the op fold and
projection computation are synchronous calls the app schedules off the UI
thread via a dedicated Web Worker owning the wasm instance. The worker is an
app-side concern (app/src/wasm); the core stays thread-agnostic. Multiple
groups may be loaded concurrently in one instance; memory ceiling concerns
ride the ADR-001 Phase 2 measurement gate.

### D6. The CLI is the same door without wasm

The `cn` CLI calls the same Rust functions natively (a thin `cn-api` facade
module inside cn-wasm's crate structure - the wasm-bindgen layer wraps the
facade, the CLI links the facade). Ingest, validate, export, and snapshot
build are facade calls; behavior divergence between app and CLI is
structurally impossible for anything the facade covers.

## Options considered and rejected

1. **wasm-bindgen class exports per entity** (Entity/Edge live objects) -
   rejected: chatty boundary, JS-held references pin wasm memory, and it
   invites the app to treat entities as mutable - violating the
   ops-only write path (ADR-002 D1).
2. **Raw memory views into the wasm heap for projections in v0** - rejected:
   couples the app to core memory layout before profiling proves need; kept
   as the D2 escape hatch behind a boundary_version bump.
3. **Binary serialization (MessagePack/CBOR/bincode) at the boundary** -
   rejected for v0: adds dependency and debugging opacity for marginal gain
   at 5k nodes; JSON keeps app, CLI, fixtures, and tests on one format.
4. **Exposing a generic query language across the boundary** - rejected:
   R6's needs (paths, neighborhoods, search, filter) are a closed set;
   a query language reopens the unfiltered-read risk D1 exists to prevent.

## Consequences

- Positive: the renderer can be replaced without touching the core (D1/D2);
  permission filtering is structurally unavoidable; CLI and app cannot
  diverge on covered behavior; every format is versioned and rejectable.
- Negative: JSON serialization cost on every projection - bounded by D3
  caching and the D2 escape hatch; measured at the Phase 2 gate.
- Negative: synchronous core calls require the app to own a worker - an
  accepted app-side complexity, kept out of the core.

## Open questions (Phase 2 may refine without a new ADR)

- Exact PathRequest/Neighborhood/Search request shapes (land with cn-graph's
  implementation, schema-versioned from day one).

## Amendments (adversarial round 1, 2026-07-06)

### A-B1. Reports are viewer-scoped

`validation_report(group_id, viewer_ctx_json)` - no report call exists
without a viewer. Report entries referencing objects outside the viewer's
projection are redacted to opaque category counts; full-detail reports
require the group's governance context; the anonymous context receives
schema-level findings only, no counts. LoadReport and SubmitReport follow
the same redaction rule.

### A-B2. Export options can only narrow

`export_snapshot` content is exactly the named viewer's projection passed
through the ADR-002 A-B5 export gate; `options_json` selects strict SUBSETS
(entity kinds, story selection, attribute categories) and can never add
disclosure classes. Independent of viewer, the export gate excludes
T3-effective values - including the owner's own - because T3 never leaves
the local store (ADR-001 A-B2, ADR-002 A-B8).

### A-B3. Honest trust scope for viewer naming

v0 is local-first and single-user: the app process is trusted to name the
viewer context, and the boundary is a CORRECTNESS boundary, not an
authentication boundary - stated as an explicit limitation, not implied.
Phase 5 (personal mode, R4) MUST add core-owned session identity: viewer
contexts above `group` bind to a session established through core-managed
credentials, specified in a Phase 5 ADR. That ADR is a declared dependency
of shipping personal mode; until then `self`/`admin` contexts exist for
development and viewer-switcher testing only.

### A-B4. Hidden is indistinguishable from absent

One error code, `NotFound`, covers both missing and not-visible for every
call; error `details` never carry ids, kinds, or attribute names the viewer
cannot see; search evaluates only over projected values. This rule is
normative for every current and future boundary call.

### A-B5. cn-api facade crate (corrects D6)

The facade is its own native-safe crate `core/crates/cn-api` (rlib, zero
wasm dependencies): the single public API over the cn-* crates. `cn-wasm`
(crate-type cdylib) wraps cn-api with wasm-bindgen behind
target-gated dependencies; the CLI links cn-api directly. This fixes the
round-1 finding that one crate cannot be both cdylib-for-wasm-pack and
rlib-for-native without gating, and keeps `cargo clippy --workspace
--all-targets` clean on native targets.

### Advisories folded in

- `viewer_fingerprint` = canonical hash over (context kind, subject person
  id, resolved trust-grant revision, role set, template version); any
  component change invalidates the cached projection.
- Monotonic revision enforcement is a stated obligation of the app state
  machine (I4): stale worker responses (lower revision than current) are
  discarded, never rendered.
- Load API is streaming-shaped from v0: `load_group_begin(group_id,
  template_json)`, `load_ops_chunk(group_id, ops_jsonl_chunk)`,
  `load_group_commit(group_id) -> LoadReport`, so large logs stream and
  validation warnings can surface progressively; v0 may implement it
  all-at-once internally without changing the API (I7-safe).
- Projection payload is split: `projection()` returns the DISPLAY projection
  (ids, kinds, labels, display-designated attributes, edges, weights,
  revision); full attribute detail comes per entity via
  `entity_detail(group_id, viewer_ctx_json, entity_id)`. The Phase 2
  measurement gate (ADR-001) gains a projection-payload budget measured at
  5k nodes / 10k edges.
