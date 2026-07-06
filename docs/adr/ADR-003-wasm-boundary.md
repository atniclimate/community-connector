# ADR-003: WASM Boundary Shape

- Status: draft - awaiting adversarial round 1
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
- LoadReport/SubmitReport field detail beyond quarantine and validation
  summaries.
