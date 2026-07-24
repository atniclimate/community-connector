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
| Verification battery (11 members) | `scripts/check-all.ps1` (+ `scripts/hooks/pre-commit`) |
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

**NOT done - ordered next actions (mandate item 2 continues autonomously):**
1. **Implement the intake pipeline per the blueprint's 11-step sequencing**
   (cn-store `append_batch_idempotent` seam FIRST, then cn-model intake
   provenance block, cn-ingest, `cn intake apply`, facade, forms, wizard,
   pii-scan tripwires, fixtures). check-all green + atomic commit per step;
   the reviewer's implementation gates (canonical digest golden vectors,
   fault injection, digest-bound tombstone reconciliation) are IN the
   blueprint's test lists. Then the MANDATORY adversarial round on the
   implementation diff.
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

1. **D-023 solo correctness pass (D-059.10):** review
   `docs/design/intake-consent-text-draft-2026-07-24.md` using its built-in
   checklist (~20 min). NEW: its section 7 lists four wording conflicts the
   ADR-005 rounds surfaced - the largest is the removal-semantics decision
   ("taken out of the network" vs the append-only log: no-longer-shown or
   true erasure - your call). Your sign-off clears the text for
   build/synthetic use; record it as a DECISIONS entry.
2. **Committee touchpoint (bundled, D-059.9/10):** when timing becomes known, put
   the collective-checkpoint ask AND the reviewed consent text to ATNI Climate
   together. Until the recording exists, August pilots remain CONDITIONAL and all
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
