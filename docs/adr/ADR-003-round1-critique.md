## Blocking

1. `validation_report(group_id)` has no `viewer_ctx_json`. Scenario: an anonymous app calls it after load and receives quarantine entries naming a private entity id, hidden kind, owner, provenance actor, or invalid hidden attribute. That is an unfiltered read path.

2. `export_snapshot(group_id, viewer_ctx_json, options_json)` is underspecified. Scenario: options include validation, provenance, ops, or full-fidelity export, and the snapshot returns suppressed references, T3 values, actor metadata, or dependency gaps. This can bypass ADR-002 A-B5 export gating and the T3 never-export rule.

3. Viewer context is app-named. Scenario: a compromised renderer calls `projection(group, {"context":"self","person_id":"victim"})` or `admin` and cn-perm treats it as authoritative. The ADR does not define core-owned session identity, local credential binding, or a Phase 5 scope limit for personal mode, so `self` and `admin` are forgeable.

4. Error and report payloads can leak hidden existence. Scenario: `query_paths` asks for a hidden endpoint id and D4 returns `{ code:"NotVisible", details:{ entity_id, kind } }` instead of making hidden indistinguishable from absent. Same risk applies to search over hidden attributes and quarantine reports surfaced through `LoadReport` or `SubmitReport`.

5. D6 overclaims one-crate CLI and wasm structure. A crate built by `wasm-pack` normally needs `crate-type = ["cdylib"]`; a native CLI needs an `rlib` to link. Also unguarded `wasm-bindgen`, `web-sys`, or JS extern code can break `cargo clippy --workspace --all-targets` for native targets. The facade should be a native-safe crate or explicitly gated with crate types and target-specific deps.

## Advisory

1. D3 does not define `viewer_fingerprint` derivation. Scenario: fingerprints omit subject id, trust scope, role, or canonicalization, causing Alice's cached projection to be reused for Bob or after a trust change.

2. Revision invalidation is app-enforced. Scenario: a revoke submit increments revision, but an older worker response arrives later and the UI renders stale private data. The boundary should require monotonic revision handling in `app/src/state/`.

3. `load_group(group_id, template_json, ops_jsonl)` is all-at-once and synchronous. Scenario: a large `ops.jsonl` copies into wasm as one string, folds for seconds in the worker, delays all calls, and cannot stream validation warnings. Chunked load or snapshot-first load should be specified.

4. JSON projection cost is not proven credible at 5k nodes and 10k edges. Scenario: a projection including attributes, tiers, and provenance serializes to multi-MB JSON, then parses and redraws the graph after each revision. D3's "redraw is cheaper than diffs" needs a Phase 2 measurement gate or narrower projection payload.

## Verdict

REDESIGN