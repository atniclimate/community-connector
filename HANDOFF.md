# HANDOFF.md - Live State (pickup target)

> This file is the /pickup target and outranks session memory. Reading order for a
> fresh session: CLAUDE.md, then this file, then PLAN_1.0.md "Current Position" +
> "Decision Gates", then current-phase ADRs. DECISIONS.md is the durable judgment
> record (through D-060; D-057..D-060 are the 2026-07-24 execution, sweep, grill,
> and first-push records). Every path below was verified on disk at the 2026-07-24
> true-up. **THE REPO IS NOW PUBLIC** - origin is
> `https://github.com/atniclimate/community-connector` and every pushed commit is
> world-readable. The pre-commit PII scan and I1 are the publication boundary.

## What this project is

Community Navigator (repo folder `community-connector`) is a local-first, privacy-first
tool for a community to see itself as a permission-filtered 3D graph of people, places,
orgs, skills, and needs, and to route a need to people/resources who can meet it. A Rust
core compiled to WASM owns data, schema, permissions, and graph queries; the TypeScript
app renders only viewer-scoped projections it is handed (it never re-implements
permission logic). The committed near-term deliverable is **v0.1.0 "convention-pilot
ready"** (ratified D-052): a facilitator-run build for the ATNI Climate convention arc -
convention 2026-09-14; August internal pilots on the normal app build are CONDITIONAL
on the recorded collective checkpoint (D-059.9).

## Where everything lives

| What | Path |
|---|---|
| Durable contract (mission, R1-R10, gates, autonomy) | `CLAUDE.md` |
| Invariants I1-I12 (review standard) | `AGENTS.md` |
| Decision register (D-001..D-060) | `DECISIONS.md` |
| Execution Plan v2 (accepted D-039; reconciled 2026-07-24) | `docs/PROJECT_PLAN.md` section 3 |
| Route map to 1.0 (v1.1, tracked) | `PLAN_1.0.md` |
| Repo inventory snapshot (descriptive, 2026-07-24) | `MANIFEST.md` |
| External-dependency audit (world-readable) | `DEPENDENCIES.md` |
| Machine-local content (NEVER commit) | `_private/` (gitignored): `DEPENDENCIES-local.md`, `PREDECESSOR-EXCLUSIONS.md` |
| Accepted architecture decisions | `docs/adr/` (ADR-001..005 ALL ACCEPTED; ADR-005 accepted 2026-07-24 after EIGHT adversarial rounds, D-061..D-068) |
| Intake-pipeline director blueprint (P3.5/P3.6) | `docs/blueprints/intake-pipeline.md` (aligned to accepted ADR-005 through round 8) |
| ADR-005 review trail (8 rounds) | `_reviews/community-connector/2026-07-24_adr-005-remote-intake*.md` (out-of-repo lane) |
| Intake/consent text package (DRAFT, pending D-023) | `docs/design/intake-consent-text-draft-2026-07-24.md` |
| Facilitator keygen ceremony design | `docs/design/facilitator-keygen-ceremony.md` |
| Facilitator role blueprint / authority matrix | `docs/blueprints/facilitator-role.md`, `docs/design/authority-matrix.md` |
| Snapshot scope + byte ledger | `docs/design/snapshot-viewer-scope.md`, `docs/design/snapshot-ledger.md` |
| Pilot FORM draft (Parts A/C SUPERSEDED) / evidence template / migration recipe | `docs/design/pilot-form-and-template-2026-07-06.md`, `docs/pilot-evidence-template.md`, `docs/cpf-rcn-migration-recipe.md` |
| Verification battery (12 members incl. pii-selftest) | `scripts/check-all.ps1` (+ `scripts/hooks/pre-commit`) |
| Rust core | `core/crates/cn-{model,schema,store,perm,graph,api,wasm}`, `core/cli` (`cn`) |
| App | `app/src/{viz,ui,state,wasm,theme}`; snapshot boot reader `app/src/state/snapshot.ts` |
| Codex adversarial artifacts | out-of-repo review lane `_reviews/community-connector/` under the dev workspace |
| Prior handoffs | `docs/archive/handoffs/` |

## State of play

**DONE and verified (check-all 11/11 green at every 2026-07-24 commit; clean tree;
remote in sync):**
- Phases 0-2 + Phase 3 facilitator role + Phase 5 slice: unchanged from the
  2026-07-24 ultracode-close archive (orchestrator/hooks, explore surface minus P1.3,
  persisted schemas, facilitator role with five-class no-leak property, CLI
  validate/export, evidence template, migration recipe).
- **R2 EntityDetail fixes (D-049 -> D-057, commit 73b5656):** effective tier over the
  projected attribute set; cn-api a pure carrier (I2); full viewer-class custody+tier
  matrix incl. e2e; one-line normalization incl. U+2028/U+2029. Fresh adversarial
  round PASS-WITH-NOTES, blockers CLOSED, residuals closed in-commit.
- **Gate-opener drafts (7afc20e):** ADR-005 remote-intake DRAFT; D-023 consent-text
  package; keygen ceremony design.
- **D-055 sweep executed (D-058, 8a58c7f):** world-readability pass, _private/ split,
  three root docs tracked, PROJECT_PLAN.md reconciled.
- **Grill rulings executed (D-059, 9a7688b):** targeted redactions (tenant email,
  exclusion-list enumerations to _private/ with pointers), THE_STORY approved,
  old form text marked SUPERSEDED, ADR-005 retention resolved (keep rejected records
  for the pilot window, then ONE RECORDED PURGE SWEEP), split push/deploy bars.
- **FIRST PUSH executed (D-060):** origin live, `main` tracking, plain `git push`
  works (repo-local gh credential pin as the atniclimate account).

**DONE 2026-07-24 (second session): ADR-005 ACCEPTED after eight adversarial
rounds** (D-061..D-068; rounds 1-7 FAIL-and-amend, round 8 PASS-WITH-NOTES;
every finding verified against files/code before judgment). Headline design
outcomes now binding: browser trust model with off-origin full-bundle pin
(D8); NATIVE durable owner - the app is create-only, `cn intake apply` owns
all mutation (D4); idempotent decision inbox (decision_generation CAS,
writeless replays, two-kind history with four transaction events); receipt
ledger with disjoint reconciliation; enforceable rotation cutoff; consent
affirmation surviving the purge sweep via the versioned intake provenance
block. The P3.5/P3.6 director blueprint is written and aligned
(docs/blueprints/intake-pipeline.md). The consent draft carries new section
7 (four wording conflicts for the D-023 pass, incl. the removal-semantics
human decision).

**Implementation position (2026-07-24, same session): blueprint steps 1-3
of 11 LANDED, check-all green at each commit:**
- Step 1 (78e9aae): cn-store `append_batch_idempotent` seam - durable
  classification, shadow preflight, RecoveryUnderIntent completing without
  re-authorization; 8 crash-simulation tests.
- Step 2 (661b02c): cn-model optional `IntakeProvenance` block on the
  envelope (own version, unknown-major rejected); model schema PATCH bump
  0.1.0 -> 0.1.1; fixed two version-space conflations the bump exposed in
  cn-api/cn-schema tests.
- Step 3 (73e228b): cn-ingest queue formats (record/sidecar, two
  counters, two-kind history, all four transaction events), decision
  admission table (generation+state CAS, writeless replays), recovery
  classification; 15 tests incl. both round-6 mandatory sequences.
- Step 4 (52e274b): near-duplicate surfacing (projection-bounded,
  reasons, conservative matching) + plan_approval (populated
  EntityCreate per D-069, intake block on entity and every attribute
  instance, pre-link batch digest, deterministic under injected ids,
  authoritative validate_entity report). 20 cn-ingest tests total.
- Step 5 (1db3cd0, D-070): `cn intake apply` - the native durable owner.
  Queue lock (fs4 OS lock = the liveness check), worktree/cloud-sync
  guard, atomic write primitive, crash-state recovery execution
  (approval recovery first, RecoveryUnderIntent - no re-authorization),
  tombstone reconciliation, deterministic admission, plan -> seam ->
  transaction events, I12 JSON run report. ApprovalPlanRef now persists
  the planned ops verbatim (recovery cannot regenerate ids). Six
  integration tests incl. the FULL decide -> apply -> reload round trip
  on synthetic data and the authority-matrix preflight denial. Queue
  file layout fixed as the step-8 FSA adapter contract (D-070.3).
- Step 6 (b47ab49, D-071): read-only cn-api/cn-wasm intake facade
  (BOUNDARY_VERSION 0.2.0) - intake_validate_record (plan-path reuse,
  report identical to apply-time), intake_dedup_check (the blueprint's
  missing dedup module now exists; five-arm verdict incl.
  transport_conflict), intake_near_duplicates (viewer projection
  computed in-core). NO approval-write export. The no-leak extension
  test passes on a real projection (trust-granted governance sees the
  hidden candidate; facilitator and anonymous never do).
- Step 7 (eaa8a13, D-072): P3.6 template-driven entry form - pure model
  consuming kinds[].attributes[] (R2), advisory-only validation, payload
  assembly with payload-carried kind; DRAFT consent boilerplate
  (D-072.1-authorized, section-7 corrections applied, DRAFT banner until
  D-023) with the structural checkbox gate (D-030).
- Step 8 (bf02e5f, D-073): FSA create-only queue adapter + intake store
  slice; cn-api/cn-wasm PURE BUILDERS intake_stage_record /
  intake_build_decision keep checksum authority in the core - the app
  never computes a digest; read-back verification on every create;
  wizard refusal rule for approved_intent/unreadable sidecars.
- Step 9 (76a0372, D-074): P3.5 facilitator wizard - dir grant + guard,
  queue dashboard (I12 surface, `cn intake apply` + reload instruction),
  entry flow, review view with the three read-only core checks, approve/
  reject-with-reason/set-aside-note/clear-failed as create-only decision
  files. DecisionType::Reject now carries its REQUIRED reason (D-074.1);
  new viewer_roles boundary export gates the mount (affordance only).
- Step 10 (d677e5d, D-075): pii-scan intake tripwires (queue path
  shapes, queue_record_version + secret-encrypted content markers with
  .rs/.ts/.md content exemption) + the pii-selftest check-all member
  (12 members) generating positive fixtures at runtime.

**NOT done - ordered next actions (mandate item 2 continues autonomously;
blueprint docs/blueprints/intake-pipeline.md section 9 is the sequence):**
1. The MANDATORY adversarial round on the WHOLE implementation diff -
   steps 1-11 are committed but unreviewed: 78e9aae, 661b02c, 73e228b,
   52e274b, 1db3cd0, b47ab49, eaa8a13, bf02e5f, 76a0372, d677e5d, plus
   the step-11 fixtures commit (all 11 blueprint steps now landed).
   Permission-adjacent at the approval boundary; the round is a
   blueprint precondition for acceptance. Deferred polish for later:
   FSA handle persistence across sessions (IndexedDB; D-074 note).
2. Asana refresh owed for this arc's notable progress (ADR-005
   acceptance + blueprint steps 1-11) per the ratified convention -
   deferred at session close for context budget; do at next close.
2. **Remote intake relay implementation (per ACCEPTED ADR-005).** Pages form,
   client-side sealed box, Workers+KV relay (receipt ledger, admission
   allowlist), pilot-PC puller (bundle+key pins, crash protocol). The
   acceptance UNLOCKS building this; the DEPLOY bar (D-059.8) still
   requires: intake pipeline working + keygen ceremony executed + D-023
   sign-off on form text.
3. **Snapshot data pipeline (D-048 / P2.3-P2.5)** - targets the convention build.
4. **Phase 4 slimmed (D-056.3):** minimal P4.1 story authoring.
5. **P1.3 benchmark** deferred to September; record in ADR-004.

## The human's queue

1. **D-023 solo correctness pass (D-059.10):** DEFERRED by the human
   (D-072) - review `docs/design/intake-consent-text-draft-2026-07-24.md`
   with its built-in checklist when ready; section 7's removal-semantics
   decision (no-longer-shown vs true erasure) is still yours. In the
   meantime engineering is AUTHORIZED to wire PLACEHOLDER/DRAFT-marked
   boilerplate matching real functionality (D-072.1); nothing community-
   facing ships without your sign-off.
2. **Committee touchpoint (D-059.9/10, timing resolved by D-072.2):** the
   collective checkpoint's moment is the CONVENTION (2026-09-14); individual
   ad hoc demo meetings beforehand can open further authorization pathways -
   demo-readiness on synthetic data is now a sequencing priority. Until a
   recorded checkpoint exists, August pilots remain CONDITIONAL and all
   engineering stays on synthetic data.
3. Standing: G2 Open Eligibility isolation if ever added; G-BACKUP still ACCEPTED,
   not solved (the public remote holds code only, never data - not a backup answer
   for ops); pilot-window close requires the RECORDED rejected-record purge sweep
   (D-059.11).

## Non-negotiables a fresh session must not violate

- **The repo is public.** Every commit that pushes is world-readable. No real-person
  PII in the repo, any commit, any fixture, or any Codex prompt (I1) - now also the
  publication boundary. Never commit `_private/` content.
- **Deploy bar is unmet** (D-059.8): nothing goes live on Pages or Workers until
  ADR-005 is accepted, the intake pipeline works, the keygen ceremony has been
  executed, and D-023 sign-off covers the form text. Cloudflare spend is approved
  for the intake relay only; no other spend.
- **No real ingestion** before the recorded collective checkpoint (D-030/D-050/
  D-059.9). The predecessor exclusion rule is absolute; the enumerated list lives
  at `_private/PREDECESSOR-EXCLUSIONS.md` (D-059.3).
- Permission logic lives only in `cn-perm` (I2); state mutates only through
  `app/src/state` (I4); provenance + tier on everything (I6); versioned formats
  with unknown-major rejection (I7); snapshot under 5MB (I8); docs use hyphens (I10).
- Verification loop before every commit; atomic per-unit conventional commits; full
  `check-all` precedes each commit series. Permission-adjacent work gets a director
  blueprint + mandatory adversarial round. ADR-005 is NOT accepted until its round runs.

## Key design commitments (shortest refresher)

- Rust/WASM core is the single source of truth; the app renders permission-filtered
  projections only. Event-sourced op log in `cn-store`; state is a fold over ops.
- Intake (D-053, ADR-005 draft): in-app entry + facilitator pending-review queue
  (OUTSIDE the op log) is the primary path; remote path is QR -> Pages form ->
  client-side sealed box -> Cloudflare ciphertext relay -> pilot-PC
  pull/decrypt/durable-stage -> relay wipe. No auth in v0.1.0/v1.0. No server ever
  holds readable personal data; the graph never listens. Approved remote entries
  land unowned (D-056.2); rejected records keep-then-recorded-purge (D-059.11).
- Snapshot targets the CONVENTION build (D-056.3); August pilots run the normal app
  build. Snapshot is NOT yet self-contained (~1.57MB external worker) - next-action 4.
- TSDF tier codes primary in the UI (D-032); in-app story authoring in v0.1 (D-037) -
  deliberate choices against recommendations; do not "fix" them.
- Codex offload (gpt-5.6-sol, D-042): the adversary wrapper is healthy (two clean
  rounds 2026-07-24); the [[codex-exec-early-exit]] caution stands for long raw
  `codex exec` jobs.
