## Resolved

A-B1: No - viewer-scoping fixes raw report access, but opaque category counts outside the viewer projection still leak hidden existence.

A-B2: Yes - export is constrained to the named viewer projection, ADR-002 A-B5, subset-only options, and T3 exclusion.

A-B3: Yes - forgeable viewer context is explicitly limited to local-first v0, with Phase 5 session identity required before personal mode ships.

A-B4: Yes - `NotFound`, safe details, and projected-only search close the hidden-existence error leak.

A-B5: No - `cn-api` exists as a native lib, but Cargo metadata shows `cn-wasm` as `lib`, not `cdylib`; no target-gated `wasm-bindgen` deps are present, so the wasm-pack claim is not true yet.

## New blocking

1. Streaming load has no viewer context: `load_group_commit(group_id) -> LoadReport` and progressive warnings from chunk loading cannot apply A-B1 redaction. Scenario: a group member imports ops containing private quarantined entities and receives validation counts or warnings for objects outside their projection.

2. `entity_detail` is underspecified. Scenario: a visible entity has a hidden T3 attribute; `projection()` omits it, but `entity_detail(..., entity_id)` returns "full attribute detail" unless explicitly constrained to the same viewer-filtered projection rules.

3. Report redaction still leaks via counts. Scenario: a non-governance viewer receives opaque counts for hidden quarantine categories and infers hidden objects or invalid hidden attributes exist, contrary to the no-count leakage rule in the permission spec.

## Verdict

ACCEPT-WITH-AMENDMENTS