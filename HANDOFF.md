# HANDOFF.md - Live State (pickup target)

> This file is the /pickup target and outranks session memory. Reading order for a
> fresh session: CLAUDE.md, then this file, then PLAN_1.0.md "Current Position" +
> "Decision Gates", then current-phase ADRs. DECISIONS.md is the durable judgment
> record (through D-056; D-050..D-056 are the 2026-07-24 gate-grill + look-back
> rulings). Every path below was verified on disk at true-up (2026-07-24).

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

**DONE and verified (check-all 11/11 green, re-verified 2026-07-24; clean tree except
the three root docs parked pending the D-055 sweep):**
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

**NOT done - ordered next actions (resequenced 2026-07-24 per D-056 for the August
pilot window; internal pilots mid-to-late August on the NORMAL app build - the
snapshot pipeline is a convention deliverable, not a pilot blocker):**
1. **R2 EntityDetail fixes (FIRST - reviewer-confirmed defects, D-049).** BLOCK-1: report
   the effective tier as max(entity.tier, effective tier over the PROJECTED attribute set),
   in cn-perm (fixes the T0-shown-beside-T2 mislabel). BLOCK-2: relocate `own_settings`
   tier/visibility disclosure + effective-tier decisions out of `cn-api/src/lib.rs:385-421`
   into cn-perm (I2). HIGH-3: extend custody/tier tests to all five viewer classes +
   inactive-governance + dual-role, plus an end-to-end `cn-api::entity_detail` governance
   test. LOW-4: normalize control whitespace in the provenance one-liner. Gate-blind;
   permission-adjacent -> needs a fresh adversarial round after the fix.
2. **Long-lead gate-openers (parallel with 1, D-056.3).** The D-055 pre-publish sweep
   (update + track `PLAN_1.0.md`, `MANIFEST.md`, `DEPENDENCIES.md`; full-repo
   world-readability pass; full PROJECT_PLAN.md revision); draft **ADR-005** "Remote
   intake: sealed-envelope relay and facilitator pending-review queue" (required scope
   in D-056.1) + its adversarial round; draft the intake-form/consent text into D-023
   human review; design the facilitator keygen ceremony (offline private-key backup,
   key-fingerprint pinning in the puller). LICENSE.md is already tracked (D-054
   precondition satisfied; re-verify at push time).
3. **P3.5 facilitator wizard + P3.6 entry forms - THE intake pipeline (D-053).**
   Direct in-app entry plus the pending-review staging queue: every submission (in-app
   or remote) lands pending and enters the graph only on facilitator approval; queue
   persistence is durable-write-first, gitignored staging, pii-scan covered, with
   near-duplicate surfacing (D-056.4). UI over existing cn-api submit/load; validation
   surfaced from cn-schema; I2/I4. Approved remote entries land unowned,
   facilitator-created (D-056.2).
4. **Remote intake relay (D-053, per ADR-005).** Static intake form (name, tribe, orgs,
   roles) for GitHub Pages; client-side sealed-box encryption to the facilitator public
   key (private key only on the pilot PC); minimal Cloudflare Workers + KV ciphertext
   relay (human's account, spend approved; payload size caps, rate limits, KV TTL);
   pilot-PC puller that decrypts locally, stages durably into the review queue, THEN
   wipes the relay. Push/deploy prereqs: license in-repo (satisfied), D-055 sweep
   passed, core stability. Localized offline intake stays the later stretch goal.
5. **Snapshot data pipeline (D-048 / P2.3-P2.5) - resequenced after intake (D-056.3);
   targets the convention build.** Main-thread `WasmTransport` so `dist/index.html`
   needs no external worker (D-046); wire `--public-layer` into `build:snapshot`
   (isolated non-tracked dir); per-artifact size gate in `check-size.mjs`; no-leak
   acceptance test (3+ above-scope sentinels absent, D-047.4); then flip
   `CN_EMBED_SNAPSHOT` on by default. Hardest piece.
6. **Phase 4 slimmed for the window (D-056.3):** minimal P4.1 story authoring under
   facilitator authority for the pilots; P4.2 primer and the bulk of P4.3 defer; P4.4
   snapshot acceptance rides item 5 - all with the mechanics/language split (D-044.4).
7. **P1.3 benchmark** deferred to September (post-pilot); re-run with
   labels+halos+motion live; record in ADR-004.

## The human's queue

**All six queued gates were answered 2026-07-24 in the gate-grill session -
see D-050..D-055.** Summary: convention 2026-09-14 with August internal pilots
(D-050); ATNI authors vocabulary post-stability (D-051); v0.1.0 ratified as the
convention-arc finish line with a real-usage acceptance bar (D-052); intake is
in-app entry + a QR sealed-envelope relay via GitHub Pages + Cloudflare Workers,
gates opened by the human (D-053); PolyForm Noncommercial 1.0.0 (D-054); the
three root docs tracked after a pre-publish sweep (D-055).

Still on the human:
1. **Collective checkpoint:** a recorded ATNI Climate committee approval must
   exist BEFORE the first internal-pilot ingestion of real people (D-030/D-050).
2. **D-023 review:** intake-form text, consent wording, and any community-facing
   language need human review before use - now concretely needed for the relay
   form and the August pilots.
3. Standing: Open Eligibility mapping isolated if ever added (G2); single-machine
   backup risk still ACCEPTED, not solved (G-BACKUP / D-026) - note the public
   remote will hold code only, not data, so it is not a backup answer for ops.

## Non-negotiables a fresh session must not violate

- **Human gates are absolute** (CLAUDE.md). Three were OPENED by the human on
  2026-07-24 with conditions (D-053/D-054/D-055): the public remote
  `atniclimate/community-connector` may be added and pushed ONLY after the
  PolyForm Noncommercial 1.0.0 license is in-repo, the pre-publish sweep has
  passed, and core stability is reached; Cloudflare Workers spend is approved
  for the intake relay only. Everything else stands unchanged: no real-person
  PII in the repo, any commit, any fixture, or any Codex prompt (I1); no other
  spend; no protocol/identity/federation commitment beyond D-053's intake path.
- Do **not** commit `PLAN_1.0.md`, `MANIFEST.md`, `DEPENDENCIES.md` until the
  D-055 pre-publish sweep unit runs (then track all three).
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
- Intake (D-053): in-app entry + facilitator pending-review queue is the primary
  path; remote path is QR -> static Pages form -> client-side sealed-box
  encryption -> Cloudflare ciphertext relay -> pilot-PC pull/decrypt/review. No
  auth in v0.1.0/v1.0. No server ever holds readable personal data; the graph
  never listens on the network.
- Codex is the offload engine (gpt-5.6-sol, D-042) but was exiting early / landing output
  LATE this session (degraded mode, CODEX_GUIDE section 8) - verify its health first, use
  small bounded jobs, and reconcile late-landing review files. See memory
  `codex-exec-early-exit`.
