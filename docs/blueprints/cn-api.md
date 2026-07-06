# Blueprint: cn-api + cn-wasm + smoke page (Phase 2 close, authored by director)

Realizes ADR-003 D1-D6 with both rounds' amendments. cn-api is the native-safe
facade (rlib, no wasm deps); cn-wasm wraps it with wasm-bindgen; a smoke page
in app/ proves the bundle loads. The cn CLI gains NO subcommands yet (Phase 4).

## cn-api dependencies

```toml
[dependencies]
cn-model = { path = "../cn-model" }
cn-schema = { path = "../cn-schema" }
cn-store = { path = "../cn-store" }
cn-perm = { path = "../cn-perm" }
cn-graph = { path = "../cn-graph" }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
semver = { version = "1", features = ["serde"] }
thiserror = "2"
```

## cn-api surface (all &str JSON in, String JSON out - ADR-003 D2)

```rust
pub struct Api { /* groups: BTreeMap<GroupId, GroupSession> */ }
// GroupSession (private): GroupState + revision: u64 + pending load buffer
// + projection cache keyed by viewer_fingerprint (ADR-003 D3).

impl Api {
    pub fn new() -> Self;
    pub fn core_info(&self) -> String;
    // {"core_version":"0.1.0","boundary_version":"0.1.0","supported_schema_majors":[0]}

    // Streaming load (ADR-003 round-2 amendment): viewer named at begin.
    pub fn load_group_begin(&mut self, group_id: &str, viewer_ctx_json: &str,
                            template_json: &str) -> String;
    pub fn load_ops_chunk(&mut self, group_id: &str, ops_jsonl_chunk: &str) -> String;
    pub fn load_group_commit(&mut self, group_id: &str, now_ms: i64) -> String;
    // commit folds buffered ops, returns LoadReport JSON REDACTED via
    // cn_perm::redact_report for the viewer named at begin.

    pub fn submit_ops(&mut self, group_id: &str, viewer_ctx_json: &str,
                      ops_json: &str, now_ms: i64) -> String;
    // authorization via cn_perm::PermAuthorizer (ADR-002 A-B6); bumps
    // revision on ANY applied op; returns SubmitReport (redacted) with
    // outcomes per op.

    pub fn projection(&mut self, group_id: &str, viewer_ctx_json: &str) -> String;
    pub fn entity_detail(&mut self, group_id: &str, viewer_ctx_json: &str,
                         entity_id: &str) -> String;
    // detail = the viewer's projected attribute set for that entity plus
    // visibility/tier ONLY for values where owner_is_viewer (own-record
    // management, R4); never anyone else's settings. Hidden or missing
    // entity -> NotFound (A-B4).

    pub fn query_paths(&mut self, group_id: &str, viewer_ctx_json: &str,
                       request_json: &str) -> String;
    pub fn query_neighborhood(&mut self, group_id: &str, viewer_ctx_json: &str,
                              request_json: &str) -> String;
    pub fn search(&mut self, group_id: &str, viewer_ctx_json: &str,
                  query_json: &str) -> String;
    // all three: build/reuse cached GraphIndex per (viewer_fingerprint,
    // revision); requests/responses are the cn-graph types serialized.

    pub fn validation_report(&self, group_id: &str, viewer_ctx_json: &str) -> String;
    pub fn export_snapshot(&mut self, group_id: &str, viewer_ctx_json: &str,
                           options_json: &str) -> String;
    // v0 options: {"kinds":[...]} subset filter only (A-B2: narrowing only).
    // Content: the projection (already tier-gated; T3 unreachable by
    // construction) + template + boundary/schema versions.
}
```

Every method returns the envelope `{"ok": ...}` or
`{"err":{"code":"...","message":"...","details":{...}}}` (ADR-003 D4).
Codes (closed enum, serialized snake_case): `not_found`, `invalid_json`,
`invalid_viewer`, `unsupported_schema_version`, `group_exists`,
`group_not_loaded`, `load_not_begun`, `denied`, `internal`.
NotFound opacity (A-B4): unknown group, hidden entity, and missing entity all
return `not_found` with NO identifying details beyond what the caller sent.
No panics anywhere; a caught invariant violation returns `internal` (I3).

Implementation notes:
- Projection cache invalidation: revision bump clears all cached projections
  and indexes for the group; fingerprint mismatch recomputes (D3).
- `now_ms` is the injected clock (cn-model rule); cn-api owns an HlcClock
  per group session for op sort keys when it CREATES ops (it does not in
  v0 - callers submit full ops; validate their sort_key group consistency).
- DTO structs live in a private `dto` module; public API is strings only.

## cn-wasm (wraps cn-api; target-gated)

```rust
#[cfg(target_arch = "wasm32")]
mod bindings {
    use wasm_bindgen::prelude::*;
    #[wasm_bindgen]
    pub struct CnApi { inner: cn_api::Api }
    #[wasm_bindgen]
    impl CnApi {
        #[wasm_bindgen(constructor)] pub fn new() -> CnApi;
        pub fn core_info(&self) -> String;
        // ... one thin wrapper per cn-api method, same names, &str/String,
        // now_ms as f64 from JS Date.now() cast to i64 ...
    }
}
```

Native build keeps compiling (rlib re-exports cn_api so `cargo test
--workspace` covers the crate; bindings module only for wasm32).

## Smoke page (app/)

- `app/smoke/index.html` + `app/smoke/smoke.ts` (plain module, no framework):
  imports the wasm-pack `pkg` via relative path, constructs `CnApi`, calls
  `core_info`, then `load_group_begin` (research-network fixture template,
  viewer Anonymous), `load_group_commit`, `projection` - renders the JSON
  results as <pre> text and a PASS/FAIL line per step.
- npm script `"smoke": "vite build --outDir dist-smoke --base ./ app-root?"` -
  keep it SIMPLE: a `vite` config addition is allowed, or serve via
  `vite dev`; acceptance is: `npm run build:smoke` produces a bundle with no
  TS errors, and a documented manual open step in the smoke README. Also add
  a plain node check `app/scripts/smoke-node.mjs` that loads the wasm pkg
  (nodejs target build or web target with manual instantiation) and runs the
  same call sequence headlessly, exiting nonzero on failure - wire as
  `npm run smoke:node`. If the web-target pkg cannot load in node cleanly,
  build a second `--target nodejs` pkg into `pkg-node/` for the script
  (document both in the README; pkg dirs stay gitignored).

## Measurement gates (ADR-001 + ADR-003 amendments)

Add `core/crates/cn-api/tests/measure.rs` with an `#[ignore]`d test
`measurement_gate`: generate a synthetic group (research-network template,
5,000 entities across kinds with ~10 attributes each, 10,000 edges), fold,
project for a member viewer, and print: fold wall time, projection wall
time, projection JSON byte size, ops JSONL byte size, and a coarse state
size proxy (serialized GroupState bytes). No assertions except completing
without error - the numbers get recorded in HANDOFF (director does this).
Deterministic generation (seeded LCG, fixed ids).

## Test obligations

1. Envelope discipline: every method returns valid `{"ok"}`/`{"err"}` JSON
   for both happy and error paths; unknown group -> not_found; malformed
   JSON -> invalid_json; op for another group -> denied or quarantine per
   store semantics (assert which, deterministically).
2. NotFound opacity: hidden entity (viewer can't see it) and absent entity
   produce IDENTICAL responses byte-for-byte for entity_detail and
   query_paths endpoints.
3. Streaming load: template + two chunks + commit equals one-shot load of
   the same ops (same projection output); load without begin ->
   load_not_begun; commit twice -> load_not_begun on the second.
4. Revision semantics: projection carries revision; applied submit bumps it
   exactly once per call with >=1 applied op; denied-only submit does NOT
   bump; stale cache never returned after bump (assert recompute by
   observing the new entity).
5. entity_detail: own record includes visibility/tier per value; someone
   else's projected record includes values only (no settings leak).
6. export_snapshot: kinds subset narrows; T3-effective values absent even
   for the owner-viewer export (A-B2); result parses and references only
   projected ids.
7. Round-trip: fixture template + synthetic ops built from cn-model
   constructors -> load -> projection -> cn-graph search/path calls through
   the api all succeed.

Verification: fmt, clippy -D warnings, test --workspace,
`cargo build --target wasm32-unknown-unknown -p cn-wasm`,
`wasm-pack build crates/cn-wasm --target web` from core/, and from app/:
`npm run typecheck`, `npm run build`, `npm run smoke:node`.
