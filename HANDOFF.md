# HANDOFF.md - Live State

> This file outranks session memory. Reading order for a new session: CLAUDE.md,
> then this file, then current-phase ADRs.

Last updated: 2026-07-06 ~05:00 Pacific, end of session 1 (a single overnight
session spanning bootstrap through Phase 1 completion, including a usage-limit
failover and resume - see DECISIONS D-007).

## Phase status

**Phase 0 - Bootstrap: COMPLETE.**
**Phase 1 - Domain model and ADRs: COMPLETE.** All acceptance criteria met:

| Criterion | Status |
|---|---|
| ADR-001 domain model | ACCEPTED (1 adversarial round, amended; D-010) |
| ADR-002 event log / network readiness | ACCEPTED (2 rounds, amended; D-013) |
| ADR-003 wasm boundary | ACCEPTED (2 rounds, amended; D-015) - cn-api facade crate added, cn-wasm is cdylib+rlib |
| Written specs | ADR-001 (entity/attribute model), docs/specs/permission-model.md, schemas/group-template.schema.json, ADR-002 (op log/sync) |
| Two contrasting synthetic templates | fixtures/templates/*.template.json, both PASS `npm run validate:templates` (app/) |

Also complete: docs/design/DESIGN_BRIEF.md revised per adversarial critique
(D-012, D-014); its critique round 2 is PARKED until the Phase 3 rendering
spike exists. PolyForm Noncommercial 1.0.0 license added (D-011).

## Next phase: Phase 2 - Rust core

Acceptance (CLAUDE.md / launch prompt section 5): cn-model, cn-schema,
cn-perm, cn-graph, cn-store implemented behind the cn-api facade with the
wasm boundary; cargo test green including the permission property test (no
projection ever leaks above the viewer's access); clippy clean; wasm bundle
builds and loads in a smoke page. Measurement gates from ADR rounds: wasm
heap at 5k entities x 10 attrs + 10k edges under 64MB; projection payload
budget measured.

### Phase 2 chain state (updated mid-phase)

DONE: wasm32 target installed; wasm-pack smoke build of cn-wasm PASSES
(cdylib+rlib + target-gated wasm-bindgen committed). Director blueprints
authored and committed for ALL five core crates:
docs/blueprints/{cn-model,cn-schema,cn-store,cn-perm,cn-graph}.md.
cn-model IMPLEMENTED and committed (15 tests; fmt/clippy/test/wasm green).

IN FLIGHT: cn-schema implementation on codex (task .codex/task-cn-schema.md,
result lands at .codex/cn-schema-result.md).

MECHANICAL CHAIN for whoever continues (each step: wait for result file,
re-run verification loop from core/ - fmt --check, clippy --workspace
--all-targets -D warnings, test --workspace, cargo build --target
wasm32-unknown-unknown -p <crate>, pii-scan - then atomic commit, then
`Start-Process pwsh -ArgumentList @('-NoProfile','-File','.codex\run-<next>.ps1') -WindowStyle Hidden`):

1. cn-schema lands -> verify/commit -> launch run-cn-store.ps1
2. cn-store lands -> verify/commit -> launch run-cn-perm.ps1 (high effort -
   it owns the Phase 2 acceptance property test)
3. cn-perm lands -> verify/commit -> launch run-cn-graph.ps1
4. cn-graph lands -> verify/commit -> launch run-cn-api.ps1 (blueprint
   docs/blueprints/cn-api.md is DONE and committed; the task implements
   cn-api + cn-wasm bindings + smoke page + measurement gate in one go;
   verification includes wasm-pack build and npm smoke:node from app/).
5. DONE: cn-api landed, verified (incl. Node wasm smoke PASS), committed.
   Measurement gate numbers (native, i5-1340P, 5,000 entities + 10,000
   edges, member viewer, 2026-07-06): fold 255ms; projection compute 133ms;
   projection JSON 2.88MB; ops JSONL 16.4MB; state JSON proxy 9.87MB.
   Assessment: within gates for v0. Watch items for Phase 3: projection
   JSON is 2.9MB per full recompute crossing the wasm boundary (D3 caching
   mitigates; D2 typed-array escape hatch if browser profiling demands),
   and wasm-side timings should be re-measured in the smoke page before
   Phase 3 relies on them.
6. DONE: closing review returned FIXES-REQUIRED (archived at
   docs/analysis/core-review-phase2-2026-07-06.md); director rulings in
   D-016; all accepted fixes applied, spot-checked, re-verified, committed.

**PHASE 2 CORE: CLOSED** (2026-07-06). cn-sync is intentionally a
placeholder - the SyncTransport trait + local adapter are Phase 5 scope per
the phase plan (D-016); ADR-002 A-B8 defines the contract they must meet.

## Next phase: Phase 3 - Frontend

Driven by docs/design/DESIGN_BRIEF.md (revised; its critique round 2 is
parked until spike evidence exists). Acceptance: CLAUDE.md phase plan
(viz + view modes, template-driven color, stories, detail panels, wizard +
profile editor, explicit state machine per I4, tsc + vite build clean, both
fixture groups load and render, R9 accessibility criteria, snapshot under
budget).

### Exact next three actions

1. THE RENDERING SPIKE (design brief section 9 item 1): prototype custom
   instanced Three layer vs three-forcegraph vs stock 3d-force-graph at
   5k nodes / 10k edges from a generated synthetic projection; acceptance
   30 FPS at DPR 1.5 thermally steady on this machine; ONE edge system.
   Deps land in app/ (three, 3d-force-graph pinned versions into
   docs/ENVIRONMENT.md). Decision recorded as ADR-004 (renderer choice).
2. Design brief critique round 2 (codex review) WITH spike numbers in hand;
   revise brief; then app state machine skeleton (I4) + wasm worker
   integration (ADR-003 D5: worker owns the CnApi instance; monotonic
   revision enforcement).
3. Template-driven theming pipeline from the brief section 5 (tokens from
   group template JSON, contrast + CVD enforcement, legend indicator) over
   both fixture templates.

Review debt to schedule (routing policy standing job): a codex review diff
pass over the accumulated core implementation commits before Phase 2 close.
Known I12 debt for that pass: cn-store `Snapshot::load` returns Ok(None) on
discard without a report channel - cn-api's load path must surface the
discard reason as a warning finding (cn-store result note, 2026-07-06).

## Open human gates

1. License variant: PolyForm NONCOMMERCIAL 1.0.0 was selected from the
   "polyform" answer - confirm, or name Internal Use / Small Business instead.
2. (Standing) no remotes, no real data, no spend without explicit instruction.

## Degraded modes / standing directives

- Codex runs use `--sandbox danger-full-access` (D-008; human said "figure it
  out" - D-011 keeps the bypass with mitigations).
- Routing policy is MANDATORY (CLAUDE.md, docs/CODEX_GUIDE.md section 7):
  Codex absorbs bulk work; effort-match grind tasks (D-014: multi-ruling doc
  revisions need gpt-5.5/medium, not gpt-5.4-mini/low).
- Atomic commits without asking (CLAUDE.md cadence).
- Re-arm a one-shot 8:00 AM local resume cron each session while the
  usage-failover directive stands (this session's is armed: job fb983a1b;
  session-only, dies with the terminal).
- Never point codex --output-last-message at a task's own artifact path.

## Warnings that must not be lost

- The design brief's second critique round is intentionally parked until the
  Phase 3 rendering spike produces evidence (D-012); do not treat the brief's
  rendering numbers as validated - they are labeled allocations.
- wasm32 target not installed yet (Phase 2 action 1).
- .codex/ is gitignored scratch; the token analysis that matters is preserved
  at docs/analysis/token-analysis-2026-07-06.md, and ADR critiques at
  docs/adr/ADR-001-round1-critique.md (rounds 2/3 critiques for ADR-002/003
  live only in .codex/ - archive them if ever needed before cleanup).
- Predecessor PII exclusion list (CLAUDE.md) applies to every Codex prompt;
  nothing is cleared.
