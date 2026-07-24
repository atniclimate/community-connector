# HANDOFF.md - Live State (pickup target)

> This file is the /pickup target and outranks session memory. Reading order for a
> fresh session: CLAUDE.md, then this file, then PLAN_1.0.md "Current Position" +
> "Decision Gates", then current-phase ADRs. DECISIONS.md is the durable judgment
> record (through D-049). Every path below was verified on disk at true-up
> (2026-07-18).

## What this project is

Community Navigator (repo folder `community-connector`) is a local-first, privacy-first
tool for a community to see itself as a permission-filtered 3D graph of people, places,
orgs, skills, and needs, and to route a need to people/resources who can meet it. A Rust
core compiled to WASM owns data, schema, permissions, and graph queries; the TypeScript
app renders only viewer-scoped projections it is handed (it never re-implements
permission logic). The committed near-term deliverable is **v0.1.0 "convention-pilot
ready"**: a facilitator-run, snapshot-first build for an ATNI Climate convention arc.

## Where everything lives

| What | Path |
|---|---|
| Durable contract (mission, R1-R10, gates, autonomy) | `CLAUDE.md` |
| Invariants I1-I12 (review standard) | `AGENTS.md` |
| Decision register (D-001..D-049) | `DECISIONS.md` |
| Execution Plan v2 (accepted, D-039) | `docs/PROJECT_PLAN.md` section 3 |
| Route map to 1.0 (untracked, P0.6) | `PLAN_1.0.md` (root, untracked) |
| Accepted architecture decisions | `docs/adr/` (ADR-001..004) |
| Facilitator role blueprint / authority matrix | `docs/blueprints/facilitator-role.md`, `docs/design/authority-matrix.md` |
| Snapshot scope + byte ledger | `docs/design/snapshot-viewer-scope.md`, `docs/design/snapshot-ledger.md` |
| Pilot FORM draft / evidence template / migration recipe | `docs/design/pilot-form-and-template-2026-07-06.md`, `docs/pilot-evidence-template.md`, `docs/cpf-rcn-migration-recipe.md` |
| Graph-networks research / integration plan | `docs/research/graph-networks-report-2026-07-06.md`, `docs/design/integration-plan-2026-07-06.md` |
| Verification battery (11 members) | `scripts/check-all.ps1` (+ `scripts/hooks/pre-commit`) |
| Rust core | `core/crates/cn-{model,schema,store,perm,graph,api,wasm}`, `core/cli` (`cn`) |
| App | `app/src/{viz,ui,state,wasm,theme}`; snapshot boot reader `app/src/state/snapshot.ts` |
| Codex adversarial artifacts | `C:\dev\_reviews\community-connector\` (strategy, triage, r2-review) |
| Prior handoffs | `docs/archive/handoffs/` |

## State of play

**DONE and verified (check-all 11/11 green; HEAD `2601616`; clean tree except the three
parked root docs):**
- Phase 0: `check-all` orchestrator + scoped pre-commit enforcement (D-043); gates recorded.
- Phase 1 explore surface (COMPLETE except P1.3): troika SDF labels (offline bundled OFL
  font), focus/motion with reduced-motion, legend, search, detail panel, flat reading
  projection.
- Phase 2 schemas: op-log/export + story + snapshot-envelope schemas; unknown-major loud
  rejection (P2.1/P2.2).
- Phase 3 facilitator role: `GroupRole::Facilitator` + authority matrix + fingerprint
  distinctness + five-class no-leak property (P3.1-P3.4, adversarial round D-045).
- Phase 5 slice: semantics-free `cn` CLI router + validate/export (P5.4-partial, D-044.2);
  pilot evidence template (P5.10); CPF-RCN migration recipe (P5.7, docs-only).

**NOT done - ordered next actions:**
1. **R2 EntityDetail fixes (FIRST - reviewer-confirmed defects, D-049).** BLOCK-1: report
   the effective tier as max(entity.tier, effective tier over the PROJECTED attribute set),
   in cn-perm (fixes the T0-shown-beside-T2 mislabel). BLOCK-2: relocate `own_settings`
   tier/visibility disclosure + effective-tier decisions out of `cn-api/src/lib.rs:385-421`
   into cn-perm (I2). HIGH-3: extend custody/tier tests to all five viewer classes +
   inactive-governance + dual-role, plus an end-to-end `cn-api::entity_detail` governance
   test. LOW-4: normalize control whitespace in the provenance one-liner. Gate-blind;
   permission-adjacent -> needs a fresh adversarial round after the fix.
2. **Snapshot data pipeline (D-048 / P2.3-P2.5).** Add a main-thread `WasmTransport` for
   snapshot mode so `dist/index.html` needs no external worker (D-046 self-contained);
   wire `--public-layer` into `build:snapshot` (isolated non-tracked dir); per-artifact
   size gate in `check-size.mjs`; no-leak acceptance test (3+ above-scope sentinels absent
   from the HTML, D-047.4); then flip `CN_EMBED_SNAPSHOT` on by default. Hardest piece.
3. **P3.5 facilitator wizard + P3.6 entry forms** (UI over existing cn-api submit/load;
   validation surfaced from cn-schema; I2/I4).
4. **Phase 4:** P4.1 story authoring under facilitator authority, P4.2 comprehension primer
   over the flat projection, P4.3 seeded synthetic stories + reveal-script doc, P4.4
   snapshot acceptance - all with the mechanics/language split (D-044.4).
5. **P1.3 benchmark** re-run with labels+halos+motion live; record in ADR-004.

## The human's queue (blocked on you - nothing here is autonomous)

1. **G-DATE / Q-A:** the ATNI convention date + registration window + consent-email lead
   time? (Sets the whole pilot calendar; near-date fallback is the default.)
2. **G1:** confirm ATNI Climate authors the capability vocabulary in its own words?
   (Default: empty HSDS-shaped structure; no settler vocabulary committed.)
3. **G-RAT:** is v0.1.0 the finish line, or do we ratify the fuller 1.0 line (Phases 6-9)?
4. **Q-B:** which form platform collects intake responses (Google/Microsoft/other)?
5. **Q-C (license):** PolyForm Noncommercial 1.0.0 stays, or change it?
6. **P0.6:** track or keep review-only the three untracked root docs (`PLAN_1.0.md`,
   `MANIFEST.md`, `DEPENDENCIES.md`)? Until you decide, a fresh session must NOT commit them.
- Standing / for review: community-facing text needs human review before use (D-023);
  Open Eligibility mapping isolated if ever added (G2); single-machine backup risk still
  ACCEPTED, not solved (G-BACKUP / D-026).

## Non-negotiables a fresh session must not violate

- **Human gates are absolute** (CLAUDE.md): no git remote / no publishing off-machine; no
  real-person PII in the repo, any commit, any fixture, or any Codex prompt (I1); no spend;
  no external protocol/identity/license commitment; the six queued gates above stay parked.
- Do **not** commit `PLAN_1.0.md`, `MANIFEST.md`, `DEPENDENCIES.md` (P0.6 is the human's).
- Permission logic lives only in `cn-perm` (I2); state mutates only through `app/src/state`
  (I4); every entity/edge carries provenance + tier (I6); persisted formats are versioned
  with unknown-major rejection (I7); snapshot stays under 5MB (I8); docs use hyphens (I10).
- Verification loop before every commit; atomic per-unit conventional commits; the
  pre-commit hook runs scoped `check-all`, a full `check-all` precedes each commit series.
- Permission-adjacent work gets a director blueprint + a mandatory adversarial round.

## Key design commitments (shortest refresher)

- Rust/WASM core is the single source of truth; the app renders permission-filtered
  projections only. Event-sourced op log in `cn-store`; state is a fold over ops.
- Snapshot-first delivery (D-024): the offline single-file HTML is the primary acceptance
  vehicle. It is NOT yet self-contained (still emits a ~1.57MB external worker) - that is
  next-action #2.
- TSDF tier codes primary in the UI (D-032); in-app story authoring in v0.1 (D-037) - both
  deliberate choices against recommendations; do not "fix" them.
- Codex is the offload engine (gpt-5.6-sol, D-042) but was exiting early / landing output
  LATE this session (degraded mode, CODEX_GUIDE section 8) - verify its health first, use
  small bounded jobs, and reconcile late-landing review files. See memory
  `codex-exec-early-exit`.
