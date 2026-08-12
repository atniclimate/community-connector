# HANDOFF.md - Live State (pickup target)

> This file is the /pickup target and outranks session memory. Reading order for a
> fresh session: CLAUDE.md, then this file, then PLAN_1.0.md "Current Position" +
> "Decision Gates", then current-phase ADRs. DECISIONS.md is the durable judgment
> record (through D-089; D-081..D-086 are the 2026-08-11 relay-implementation
> records, D-086 the claim-verifier review dispositions, D-087 the true-up's
> pii-scan archived-handoff exemption, D-088 the step-9 timestamp-reconciliation
> fix, D-089 the steps-1-11 adversarial round acceptance). Every path below was
> verified on disk at the 2026-08-11 relay close-out.
> **The repo is PUBLIC** - origin is
> `https://github.com/atniclimate/community-connector`. The local `main` is
> **~30 commits ahead of origin and UNPUSHED** (all relay-implementation work,
> steps 1-11 + the D-088 fix + the D-089 adversarial-round fixes + doc true-ups;
> run `git rev-list --count origin/main..main` for the exact count); pushing is
> authorized and safe but has not been run this arc. The pre-commit PII scan and
> I1 are the publication boundary.

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
| Decision register (D-001..D-089) | `DECISIONS.md` |
| Execution Plan v2 (accepted D-039; reconciled 2026-07-24) | `docs/PROJECT_PLAN.md` section 3 |
| Route map to 1.0 (v1.1, tracked) | `PLAN_1.0.md` |
| Repo inventory snapshot (descriptive, 2026-07-24) | `MANIFEST.md` |
| External-dependency audit (world-readable) | `DEPENDENCIES.md` |
| Machine-local content (NEVER commit) | `_private/` (gitignored): `DEPENDENCIES-local.md`, `PREDECESSOR-EXCLUSIONS.md` |
| Accepted architecture decisions | `docs/adr/` (ADR-001..005 ALL ACCEPTED; ADR-005 accepted 2026-07-24 after EIGHT adversarial rounds, D-061..D-068) |
| Intake-pipeline director blueprint (P3.5/P3.6) | `docs/blueprints/intake-pipeline.md` (aligned to accepted ADR-005 through round 8) |
| Intake-relay director blueprint (remote half) | `docs/blueprints/intake-relay.md` (2026-08-11, implements ADR-005 D2/D3/D6/D8) |
| ADR-005 review trail (8 rounds) | `_reviews/community-connector/2026-07-24_adr-005-remote-intake*.md` (out-of-repo lane) |
| Intake/consent text package (DRAFT, pending D-023) | `docs/design/intake-consent-text-draft-2026-07-24.md` |
| Facilitator keygen ceremony design | `docs/design/facilitator-keygen-ceremony.md` |
| Facilitator role blueprint / authority matrix | `docs/blueprints/facilitator-role.md`, `docs/design/authority-matrix.md` |
| Snapshot scope + byte ledger | `docs/design/snapshot-viewer-scope.md`, `docs/design/snapshot-ledger.md` |
| Pilot FORM draft (Parts A/C SUPERSEDED) / evidence template / migration recipe | `docs/design/pilot-form-and-template-2026-07-06.md`, `docs/pilot-evidence-template.md`, `docs/cpf-rcn-migration-recipe.md` |
| Verification battery (12 members incl. pii-selftest) | `scripts/check-all.ps1` (+ `scripts/hooks/pre-commit`) |
| Rust core | `core/crates/cn-{model,schema,store,perm,graph,api,wasm}`, `core/cli` (`cn`) |
| Remote-intake relay code (steps 1-11 COMPLETE, D-089) | keygen CLI `core/cli/src/intake/{keygen,keymat,fingerprint,selftest,backup}.rs`; crypto binding `core/crates/cn-ingest/src/{crypto,envelope,dedup,reconcile,consent}.rs`; puller + D8 verify `core/cli/src/intake/{pull,bundle}.rs` (+ tests `core/cli/tests/intake_{pull,pull_cli,bundle}.rs`); Worker `relay/`; Pages/GitHub-Pages form `form/`; form-to-graph e2e `scripts/e2e-remote-intake.ps1` + `scripts/e2e/` + `core/cli/examples/emit-approve-decision.rs`; runbooks `docs/runbooks/{e2e-remote-intake,intake-relay-deploy}.md` |
| Relay orchestration ledger (live per-step status) | project memory `relay-orchestration-state` + "State of play" below |
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

**DONE 2026-07-25: the MANDATORY adversarial round ran and the
implementation is ACCEPTED** (D-076..D-080; rounds 1-4 FAIL-and-amend,
round 5 PASS-WITH-NOTES; amendment commits d1b5f47, 9630d1b, 12c91f8,
ca4e714, 4d1aabf, fbfced3; review trail of five files in the
out-of-repo lane). Blueprint step 9 was amended in place (D-078). The
human rulings D-072 landed the same arc (consent boilerplate authorized
as draft; checkpoint = convention; ad hoc demos open pathways).

**NOT done - ordered next actions:**
1. **Deferred engineering debts (D-089 + D-080), grouped by the milestone
   that needs them:**
   - **Before DEPLOY (D-059.8):** a REAL-BROWSER smoke of the built form
     (the R4-1/R4-2 defects escaped every automated gate because no one ran
     the form in a browser) - this also discharges the D-080 standing debt
     (the live-browser rehearsal of the full intake flow through the real app,
     not just unit tests); and author the GitHub Pages deploy workflow.
   - **Before ingesting conflict-prone data:** R2-2 durable conflict-twin
     link (the wizard could double-admit unlinked conflict twins; R2-1
     already closed the unbounded-restaging half).
   - **Recorded limitations, NOT blocking synthetic work:** R1-5 Argon2
     m_cost clamp, R1-6 Argon2 doc reconcile, R4-5 localhost build guard,
     F12 real-HTTP test debt, R5-3 review-view consent line, F3 single-key
     rotation (D-086), tripwire source/archive bypass hardening (optional).
   Prior app-side browser IDB/FSA unit tests already LANDED (476116b,
   2026-08-10; 121 app tests) - the gap is the live-browser path above.
2. **Remote intake relay - DONE + ACCEPTED (D-089); kept here as the record.**
   Built per ACCEPTED ADR-005 from the director blueprint
   (docs/blueprints/intake-relay.md, 2026-08-11): 11
   steps in 5 phases (A crypto foundation, B envelope formats, C relay
   infrastructure, D puller I/O, E cross-cutting). Covers sealed-box
   Rust binding + cross-impl test vectors, keygen CLI commands, outer/
   inner envelope types, Pages form build pipeline under `form/`,
   Cloudflare Worker under `relay/`, `cn intake pull` CLI command with
   bundle verification (D8), reconciliation, and deploy runbook draft.
   Permission-adjacent: gets a MANDATORY adversarial round. The DEPLOY
   bar (D-059.8) still requires: intake pipeline working + keygen
   ceremony executed + D-023 sign-off on form text.
   PROGRESS (2026-08-11, all unpushed on main, ~30 commits ahead of
   origin incl. true-ups + the D-089 fixes; live per-step ledger = memory
   relay-orchestration-state):
   **ALL 11 STEPS LANDED + ADVERSARIAL ROUND DONE (D-089,
   ACCEPT-WITH-FIXES). Phases A+B+C+D (steps 1-8) + Phase E steps
   9-11 DONE, reviewed, remediated - the remote-intake relay is
   COMPLETE and accepted. See the round block below for the fix
   commits and deferred items.**
   Phases A+B+C (steps 1-6):
   - Step 1 sealed-box crypto binding (14b9778, D-081)
   - Step 2 keygen ceremony CLI (4a0b8db)
   - Step 3 envelope formats (69fc557)
   - Step 4 puller core logic (2df3fdd, D-082)
   - Step 5 Worker relay in relay/ (3975a77, D-084): 26 files, 43 tests
   - Step 6 Pages form in form/ (d0f6c6a, D-083): template-driven,
     libsodium sealed-box, D8 manifest pipeline, 42 tests
   Phase D (steps 7-8, the biggest step - the puller):
   - Steps 7+8 `cn intake pull` CLI + D8 bundle verification (13b8879,
     D-085): ureq HTTP client in the CLI crate ONLY (D1 fence proven by
     cargo tree - no network/tls crate in any cn-* crate); config parse,
     preconditions, main loop (receipts -> fetch -> transport-dedup ->
     fingerprint -> STANDARD-base64 decode -> open -> consent -> semantic-
     dedup -> stage SubmissionSource::Remote -> delete-after-verified-stage),
     D6 reconciliation, I12 report. HTTP injected behind RelayHttp + a
     fetch closure; 18 injected-core tests + 3 date-parser units.
   - Steps 7+8 REVIEWED by an independent claim-verifier: every headline
     claim re-ran green; it surfaced 5 undisclosed issues. FIX-NOW BUCKET
     landed as atomic commits (check-all green at each): F1 precondition
     order (c39fc70), F2 D6 orphan-blob flagging (702af71), F11 timestamp-
     parse diagnostic (a2927cb), F6 configurable envelope cap (283ecd4),
     F10 HTTP timeouts (bdc17d3), F4/F5 conflict + reconciliation + CLI-
     entrypoint tests (8db4ad5), D-086 dispositions (f92e853). cn crate
     52 -> 86 tests. F3 (rotation old-key catch-up) recorded as a LIMITATION
     not built (D-086) - binding D3 cutoff stands; multi-key puller design
     owed before any real rotation.
   Check-all green at every commit; Sonnet 5 reviews confirmed steps 1-6,
   the claim-verifier confirmed 7-8.
   ADVERSARIAL-ROUND BACKLOG: RESOLVED by the D-089 round (all these items
   were re-probed by the 5-reviewer panel; the real ones were fixed or
   dispositioned - see D-089 and the round block below). Nothing HELD remains.
   **Step 9 DONE (2026-08-11, commits 897b568 + 4bc00a4).** The form-to-graph
   e2e harness (built by the prior post-true-up session, left uncommitted;
   finished + landed this session) PASSES end to end: seal via the form's real
   crypto -> POST to a local `wrangler dev` relay -> `cn intake pull` over REAL
   HTTP (partially closes F12) -> approve (opt-in `emit-approve-decision`
   example) -> `cn intake apply` -> `cn export` -> entity visible. It is
   ADDITIVE and opt-in (NOT check-all; needs Node/libsodium + a running Worker):
   `scripts/e2e-remote-intake.ps1` (+ `.sh`, `scripts/e2e/*.mjs`,
   `core/cli/examples/emit-approve-decision.rs`), with the manual browser
   Verified-bundle counterpart at `docs/runbooks/e2e-remote-intake.md`.
   **It caught a REAL shipped defect (D-088):** the durable owner (`approval.rs`)
   demanded integer timestamps while the remote form emits ISO strings, so no
   remote submission could be approved (and the consent instant would zero).
   Fixed in 897b568 - `approval.rs` now coerces both conventions; the ISO parser
   moved to `cn-model::parse_iso8601_utc_to_unix_ms` (shared with the puller);
   cn-ingest regression test added. RESIDUAL from D-088 (folded into the
   adversarial backlog): the app-side facilitator review view renders a remote
   record's timestamps display-only and was NOT exercised by the e2e - check it
   handles ISO strings.
   **Steps 10-11 DONE (2026-08-11, commits cc5cc2c + 7709ec7 + f6e9b1c).** Step 10:
   deploy runbook DRAFT `docs/runbooks/intake-relay-deploy.md` (NOT to execute until
   the D-059.8 deploy bar clears; verified accurate vs relay/src/env.ts + ADR-005
   D6). Step 11: MANIFEST.md + DEPENDENCIES.md trued up for the whole relay surface
   (DEPENDENCIES correctly keeps registry packages out of its path-reference scope,
   points to blueprint section 9; self-containment verdict unchanged + re-verified).
   Follow-up f6e9b1c aligned NOTICE-third-party license status to D-054. Both steps
   produced by community-connector subagents, conductor-verified on disk + committed.
   Two pre-deploy gaps surfaced by step 10: the Vite `base` gap is now FIXED (R4-2,
   base=/community-connector/); STILL OWED is authoring the GitHub Pages deploy
   workflow (deploy-time), plus the D-089 real-browser smoke gate (below).
   **MANDATORY ADVERSARIAL ROUND DONE (2026-08-11, D-089): ACCEPT-WITH-FIXES.**
   Five read-only reviewers (crypto/keygen, puller/bundle, relay, form, durable-
   owner+D-088+app+e2e), refute-mandate, every material finding conductor-verified
   on disk. No confidentiality/PII blocker; every security-critical core held
   (crypto vectors 5/5, zero-trust relay, D1 fence empirically clean via cargo tree,
   permission model + durable owner). ~22 findings; all clear ones FIXED as four
   atomic commits + 2 conductor follow-ups: fbcab71 crypto/keygen (R1-1 panic->typed,
   R1-2 zeroize, R1-3 stderr TTY guard, R1-4 truthful remove_file), bd689c1 puller
   (R2-1 unbounded-PII-restage dedup fix, R2-3 cursor halt, F7 cap guard, R5-1
   impossible-date reject), 0ab9e5f relay (R3-1..3-6 I3 loud-fail + ledger schema
   version + pagination + CORS-on-error + ArrayBuffer), 5b12060 form (R4-1 CSP
   wasm-unsafe-eval, R4-2 base=/community-connector/, R4-4 vectors label, R4-3/R5-2
   tests). check-all 12/12, relay 47/47, form 47/47.
   ROUND DECISIONS: R4-2 target = GitHub Pages project subpath (base set), NOT going
   public yet; R2-2 (conflict twins not durably linked -> possible wizard double-
   admit) DEFERRED as a design follow-up (anomaly path; R2-1 closed the restaging
   half). NEW PRE-DEPLOY GATE (D-089): a REAL-BROWSER smoke of the built form is
   owed before D-059.8 can clear - the two form defects escaped every automated
   gate because no one ran the form in a browser.
   NEXT: the remote-intake relay (steps 1-11) is COMPLETE + accepted. Remaining
   relay debts are deferred/limitation (see the human's queue). Project-level next
   actions resume below (snapshot pipeline, Phase 4 story authoring). The DEPLOY bar
   (D-059.8) stays separate and unmet (keygen ceremony + D-023 + real-browser smoke).
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
