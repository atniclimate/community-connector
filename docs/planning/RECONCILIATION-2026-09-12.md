# Reconciliation: HANDOFF.md / PLAN_1.0.md against the 2026-09-12 pickup scouts

Session date 2026-09-12. This file is the first file in `docs/planning/` (created by
this session) and does not execute, and is not, any MANIFEST reorganization proposal.
Nothing in `HANDOFF.md`, `PLAN_1.0.md`, `CLAUDE.md`, `DECISIONS.md`, or any other repo
file was modified to produce this document. This document contains no recommendations
- it judges claims against evidence and stops there.

## Evidence-class note (BOOTSTRAP MODE - read this before any verdict below)

**There is no `TRACE.yaml` for this repo yet.** `project-trace` has not been run.
The evidence layer for this reconciliation is: (1) four scout reports written this
session - `_private/scratch/pickup-2026-09-12/scout-{core,app,intake,docs}.md`
(read-only, gitignored scratch, each capped at 25 tool calls) - trusted as-is except
where two scouts conflicted or a claim was load-bearing for a verdict, in which case
this session spot-checked the file directly (each such check is cited by path:line
below, distinct from a scout citation); (2) the director's own ground truth for this
session (HEAD `375e8b2`, committed 2026-08-11T17:04:41-07:00, tree clean - reconfirmed
this session via `git status --porcelain=2 -uno`, no output; `git rev-list --count
origin/main..main` = 31; `check-all` run this session = 11 of 12 PASS; toolchain
`cargo`/`rustc` 1.98.1, `wasm-pack` 0.15.0, `node` 24.19.0, `npm` 12.0.2, no `wrangler`
on PATH but 4.120.1 in `relay/node_modules`), cited below as "director ledger,
2026-09-12"; and (3) `_private/scratch/pickup-2026-09-12/check-all.log` (882 lines),
read directly by this session for several receipts. This is a materially weaker
evidence class than a trace: there are no declared `test_roots`, no import-edge graph,
and no per-task file attachments, so the sections below that depend on those (orphan
files by in-degree, a dependency-order graph) are marked not applicable rather than
faked. `_private/PREDECESSOR-EXCLUSIONS.md` and `C:\dev\CPF-RCN_demo` were not read,
per instruction.

### Arithmetic note - three commit-ahead counts, reconciled

Three documents state three different "ahead of origin" counts for the same repo
history: HANDOFF.md:13 and :191 say "~30"; `docs/NEXT_SESSION.md:23` says "18";
`git rev-list --count origin/main..main` (director ledger, reconfirmed this session)
says **31**. These are not in conflict once dated: NEXT_SESSION.md's brief is
timestamped "refreshed 2026-08-11, post-7-8 + fix-now bucket" (NEXT_SESSION.md:7) -
an earlier checkpoint the same day, before steps 9-11 and the D-089 round landed more
commits; HANDOFF.md's "~30" is the same day's end-of-day count and is the closer,
later approximation. 31 is correct as of this session; "~30" is a defensible
approximation of it; "18" is stale by design (superseded within the same day).

---

## Finding 1 - HANDOFF's own "Where everything lives" catalog omits files it elsewhere depends on

`HANDOFF.md`'s path table (HANDOFF.md:31-58) is the durable pointer a fresh session is
told to trust first. Two of its rows are demonstrably incomplete against the tree:

- **Rust core row** (HANDOFF.md:53): `core/crates/cn-{model,schema,store,perm,graph,api,wasm}`
  names 7 of the 9 workspace members. `cn-sync` and `cn-ingest` are absent from the
  list, though both are real workspace members (`core/Cargo.toml` lines 3-14, scout-core
  section 1) and `cn-ingest` specifically is the crate the relay/intake work (steps
  1-11) is built on.
- **Remote-intake relay row** (HANDOFF.md:54): names the keygen family
  (`keygen,keymat,fingerprint,selftest,backup`) and the puller (`pull,bundle`) under
  `core/cli/src/intake/`, but not `core/cli/src/intake/apply.rs` - the file implementing
  `cn intake apply`, which HANDOFF's own prose cites as the "native durable owner" no
  fewer than four times (HANDOFF.md:115, 122, 240, 245) - nor `queue.rs` (scout-core
  section 7). `core/cli/src/export.rs` and `core/cli/src/validate.rs` are similarly
  unlisted anywhere in the table despite MANIFEST.md:18 recording a "Phase 5 CLI slice
  (validate/export)" as already done.

This is a structural gap in the catalog, not a one-off typo: two independent rows each
under-enumerate their own directory by a similar margin (2 of 9; 2+ files of the
central intake surface), which is the shape of a table that was hand-written once and
not mechanically re-derived from `ls` at each true-up.

## Finding 2 - relay/ and form/ test suites: not measured this session, not measurable by check-all at all

D-089 (DECISIONS.md:2291) and HANDOFF.md:273 both close with "check-all 12/12, relay
47/47, form 47/47" as of 2026-08-11. `scripts/check-all.ps1` has 12 members (HANDOFF.md:52,
confirmed structurally by this session's own check-all run: `_private/scratch/pickup-
2026-09-12/check-all.log` shows the rust-clippy member, fixture/schema validation, a
wasm-pack build, node smoke tests, the snapshot build, a PII scan, and a PII self-test
member - all consistent with "12 members" - and none of those 12 is a relay or form test
run). Neither `relay/` nor `form/` has a vitest run in `check-all.log`, and no scout ran
`npm test` inside either directory (scout-intake's scope was read-only inspection of
config/routes, not test execution; scout-app's scope was `app/`). **This is "could not
have looked," not "looked and found nothing":** nobody - no scout, no director command
this session - invoked `npm test` in `relay/` or `form/`, so this session holds zero
evidence, positive or negative, about whether 47/47 and 47/47 still hold on the current
tree. The D-089 claim is not contradicted; it is simply outside every evidence source
available to this reconciliation.

---

## 5. Verdict per task

Taxonomy applied: `verified_done`, `verified_externally_only`, `claimed_not_verified`,
`partial`, `not_started`, `structurally_untestable`. Superseded tasks receive no verdict.

**Methodology note on stretching the taxonomy to inventory/state claims.** The six
verdicts were built for software-task "done" claims. Several rows below are inventory
claims (a path list, a crate list, a byte count) rather than task-completion claims.
Applied to those: `verified_done` = the claim matches the tree/git within any hedge it
carries; `partial` = right in part, stale or incomplete in another part; `claimed_not_
verified` = a checkable "done"/"passing" assertion this session found no supporting
run for, though the tooling to check it exists; `not_started` = an asserted absence that
the tree confirms is still true. No row below uses `verified_externally_only` or
`structurally_untestable` except the deploy-bar table, where the taxonomy fits directly
(a human, off-repo act with no in-repo evidence by design).

### Cross-cutting contradictions found while assigning verdicts

1. **CLAUDE.md's "3d-force-graph + Three.js to start" (CLAUDE.md:66) vs. ADR-004's
   binding decision** (`docs/adr/ADR-004-renderer.md:35-37`: "No 3d-force-graph link
   stack anywhere in product code... not runtime dependencies of the product app...
   devDependencies of the spike only"). Both are live, tracked, accepted documents.
   CLAUDE.md's own rule (CLAUDE.md:60: "change only via ADR plus one adversarial Codex
   round") names ADR-004 as the only legitimate mechanism for changing this stance -
   so ADR-004 governs the actual architecture, and CLAUDE.md's line 66 is prose that
   was never back-ported after the ADR that superseded it. This is confirmed structurally,
   not just by precedence: `app/package.json` devDependencies list `3d-force-graph` and
   `three-forcegraph` (scout-app section 4-5), and no production import of either was
   found in `app/src` (scout-app section 5, grep excluding `app/spike/`).
2. **HANDOFF.md's own reading-order rule** (HANDOFF.md:3-4: "CLAUDE.md, then this file")
   names CLAUDE.md as read first, yet on the renderer question CLAUDE.md is the stale
   side and HANDOFF/ADR-004 the accurate side. Unverified which the fresh-session
   reader is meant to trust on points where the two disagree; this reconciliation cannot
   settle a precedence question the repo's own rule does not address.

### A. Drift - HANDOFF.md / PLAN_1.0.md "Current Position" claims vs. this session's evidence

| # | Claim | Source | Scout/session evidence | Verdict |
|---|---|---|---|---|
| 1 | "~30 commits ahead of origin and UNPUSHED" | HANDOFF.md:13,191 | `git rev-list --count origin/main..main` = 31 (director ledger, reconfirmed this session) | `verified_done` (within its own "~" hedge) |
| 2 | "check-all 12/12, relay 47/47, form 47/47" (D-089, 2026-08-11) | HANDOFF.md:273; DECISIONS.md:2291 | This session's check-all = 11/12 PASS, rust-clippy FAIL at `core/cli/src/intake/keymat.rs:258:39` (`useless_borrows_in_formatting`, check-all.log:107-117) under rustc/cargo 1.98.1 with no `rust-toolchain.toml` found anywhere - a toolchain-drift failure on a tree unchanged since the 2026-08-11 green run (director ledger), not a regression. relay/form suites: see Finding 2 - not run this session by anyone. | `claimed_not_verified` (this session; see Finding 2 for why) |
| 3 | "Snapshot is NOT yet self-contained (~1.57MB external worker)" | HANDOFF.md:338 | Qualitative claim TRUE: `app/src/main.ts:79-80` - "The snapshot build never mounts it (read-only artifact with no worker)"; `check-all.log:878` - `dist/worker-CtUIZEL0.js` is a separate 2,107.32 kB file, external to `dist/index.html` (753.69 kB, check-all.log:877). Number is stale: actual external worker is ~2.06 MB, not ~1.57 MB (+~34%). | `partial` |
| 4 | "Where everything lives" - Rust core row lists 7 of 9 crates (omits cn-sync, cn-ingest) | HANDOFF.md:53 | `core/Cargo.toml` lines 3-14 (scout-core section 1): 9 crates + `cli` = 10 members, all present on disk | `partial` (Finding 1) |
| 5 | "Where everything lives" - App row: `app/src/{viz,ui,state,wasm,theme}` | HANDOFF.md:56 | `ls app/src` (scout-docs 4e): `main.ts state/ theme/ ui/ vite-env.d.ts viz/ wasm/ window.d.ts` - matches | `verified_done` |
| 6 | "Where everything lives" - relay code row omits `apply.rs`, `queue.rs` | HANDOFF.md:54 | scout-core section 7; both files present, `apply.rs` implements the "native durable owner" cited 4x elsewhere in HANDOFF | `partial` (Finding 1) |
| 7 | "`app/src/ui/` is a README only" | PLAN_1.0.md:82 | `app/src/ui` actually holds 1,261 lines: `search.ts`(208) `detail.ts`(216) `flat.ts`(115) `format.ts`(266) `dom.ts`(36) `ui.css`(378) plus `forms/` and `intake/` subdirs and tests (scout-app section 1); corroborated independently by `MANIFEST.md:25` ("`app/src/ui/` now contains detail, search, flat-projection, and formatting modules with tests"). PLAN_1.0.md itself flags this inventory as unaudited (PLAN_1.0.md:121-123: "the inventory above is the 2026-07-11 snapshot and was not re-audited by this reconciliation"). | `partial` (claim stale and self-disclaimed; underlying UI work is substantially further along than claimed) |
| 8 | "cn-ingest and cn-sync are placeholder library files" | PLAN_1.0.md:95-96 | `cn-sync`: confirmed still placeholder, Phase 0 scaffolding only, 0 tests (scout-core sections 2,6). `cn-ingest`: NOT a placeholder - 10 source modules (approval, consent, crypto, decision, dedup, envelope, near_dup, reconcile, record, recovery, version) and 76 tests (scout-core section 6); HANDOFF's own step 3/4/5/6 narrative (HANDOFF.md:106-132) describes it being built out in detail. | `partial` (half the claim confirmed, half contradicted) |
| 9 | "No importer, validate, export, or snapshot command" | PLAN_1.0.md:96-97 | `core/cli/src/{export.rs,validate.rs}` exist (scout-core section 7); MANIFEST.md:18 records a "Phase 5 CLI slice (validate/export)" as done; `core/cli/src/intake/{apply,pull,bundle}.rs` are the import-side commands. | `partial` (claim was accurate when written; contradicted now, and PLAN_1.0.md flags itself as unaudited at the same lines cited in row 7) |
| 10 | "`schemas/` contains only `group-template.schema.json` and `theme-tokens.schema.json`" | PLAN_1.0.md:98 | `ls schemas` (this session): `group-template.schema.json`, `theme-tokens.schema.json`, `story-path.schema.json`, `op-log.schema.json`, `snapshot-envelope.schema.json`, `README.md` - 5 schema files now, not 2. | `partial` |
| 11 | Repo-layout crate list omits `cn-api` | CLAUDE.md:88 | Lists `cn-model cn-schema cn-perm cn-graph cn-store cn-sync cn-ingest cn-wasm` (8); `core/Cargo.toml` has 9 crates incl. `cn-api` (scout-core section 1); CLAUDE.md itself references `cn-api` elsewhere (scout-docs 4b: lines 70, 125) | `partial` |
| 12 | "Rendering stays in TypeScript (3d-force-graph + Three.js to start)" | CLAUDE.md:66 | Contradicted by the binding ADR-004 (see Cross-cutting contradiction 1); no production `3d-force-graph` import found in `app/src` (scout-app section 5) | `partial` (stale prose, superseded in substance by ADR-004) |
| 13 | "Local `main` is 18 commits ahead of origin, unpushed" | docs/NEXT_SESSION.md:23 | See Arithmetic note above; accurate at its own mid-day 2026-08-11 checkpoint, stale against today's 31 | `partial` |

### B. Deploy-bar checklist (D-059.8, as amended by D-089)

| Item | Code exists? (path:line) | Human step? | Blocker? | Evidence |
|---|---|---|---|---|
| ADR-005 accepted | Yes | No | No | `docs/adr/ADR-005-remote-intake.md:3` "ACCEPTED 2026-07-24 (D-068)"; DECISIONS.md D-061..D-068 |
| P3.5/P3.6 pipeline working | Yes (built) | No | No | `form/package.json`, `relay/wrangler.toml`, `relay/package.json` (scout-intake section 4); HANDOFF.md steps 6-9 (:204-206, 231-249). Not independently re-run this session (Finding 2). |
| Keygen ceremony EXECUTED | Yes (`core/cli/src/intake/keygen.rs`; `mod.rs:44-80` exposes `keygen`, `fingerprint`, `selftest`, `backup verify`, `pull`) | Yes - offline, off-repo by design | No | `mod.rs:78`: "keygen family makes no network calls and writes no secret material"; no `public.json`/`secret.json`/backup files committed (scout-intake grep). Absence of in-repo ceremony artifacts is the design, not a gap. |
| D-023 sign-off on form text | Partial (draft text exists, not yet reviewed) | Yes - PENDING | **Yes** | `docs/design/intake-consent-text-draft-2026-07-24.md:1-7`: "DRAFT - COMMUNITY-FACING TEXT - NOT FOR USE until D-023 human review is recorded... must NOT be referenced from any live UI... until the D-023 review sign-off is recorded in DECISIONS.md." `DECISIONS.md:250` "D-023 (2026-07-06)" is the original design-direction decision (taxonomy, field principles), not a sign-off entry; no later D-023.x sign-off decision was found between D-080 and D-089. |
| Real-browser smoke of the built form | No | Implicit (someone runs a browser) | **Yes** | Grep for playwright/puppeteer/chromium across `form/`, `scripts/`: no matches (scout-intake section 1e). `app/package.json` lists `@playwright/test` 1.61.1 as a devDependency (confirmed this session) and `.playwright-cli/*.yml`/`.log` artifacts dated 2026-07-06 exist (confirmed this session, 8 files) - the tool has been used before, but no automated test implements the manual runbook at `docs/runbooks/e2e-remote-intake.md`. |
| GitHub Pages deploy workflow | No | N/A | **Yes** | `.github` does not exist at repo root (confirmed this session, glob returned no match) |

### D-089 deferred items (recorded LIMITATIONS - none blocks the deploy bar)

| Item | Blocks deploy? | Source |
|---|---|---|
| R1-5 Argon2id m_cost unclamped on read | No | DECISIONS.md:2263-2264 |
| R1-6 Argon2 doc-vs-impl params divergence | No | DECISIONS.md:2264-2265 |
| R4-5 no build guard against shipping localhost relay origin | No | DECISIONS.md:2265-2266 |
| F12 real-HTTP redirect/timeout/non-200 covered only by mocks (test debt) | No | DECISIONS.md:2266-2267 |
| R5-3 review view doesn't display consent evidence directly | No | DECISIONS.md:2267-2268 |
| F3 single-key puller can't drain pre-cutover envelopes post-rotation | No | DECISIONS.md:2268-2269 (D-086) |
| R2-2 conflicting records staged but not durably linked (possible wizard double-admit) | No (anomaly path; owed before ingesting conflict-prone data) | DECISIONS.md:2255-2261 |

### C. v0.1.0 remaining work, re-verified

| Item | Status claimed | Verdict | Evidence |
|---|---|---|---|
| Pre-deploy gate: real-browser smoke + GitHub Pages workflow | NOT done (HANDOFF.md:165-169) | `not_started` | See deploy-bar table above - both confirmed absent this session |
| R2-2 durable conflict-twin link | NOT done, deferred (HANDOFF.md:170-172) | `not_started` | DECISIONS.md:2255-2261, "DEFERRED as a design follow-up" - no fix commit named |
| Recorded limitations (R1-5, R1-6, R4-5, F12, R5-3, F3) | NOT done, not blocking synthetic work (HANDOFF.md:173-176) | `not_started` | DECISIONS.md:2263-2269 - recorded, unfixed |
| Remote intake relay (steps 1-11) | DONE + ACCEPTED, "kept here as the record" (HANDOFF.md:179) | `verified_done` | DECISIONS.md D-089 (2201-2292) records ACCEPT-WITH-FIXES; tree unchanged since (director ledger). The embedded test tallies within this claim are `claimed_not_verified` this session - see drift row 2/Finding 2; the acceptance event itself is not in question. |
| Snapshot data pipeline (D-048 / P2.3-P2.5) | NOT done (HANDOFF.md:284) | `partial` | `CN_EMBED_SNAPSHOT` gate exists (`app/vite.config.ts:32-33`, director ledger receipt); `npm run build:snapshot` runs and produces `dist/index.html` at 753.69 kB / 0.72 MB, under the 5 MB budget (check-all.log:866-880) - but the worker stays external (2,107.32 kB, check-all.log:878) and is never mounted in the snapshot build (`app/src/main.ts:79-80`), so the pipeline runs but does not yet produce a self-contained artifact. |
| Phase 4 slimmed: minimal P4.1 story authoring | NOT done (HANDOFF.md:285; D-056.3) | `not_started` | `PLAN_1.0.md:756` carries it as an unchecked box: "`[ ]` P4.1 In-app story authoring + viewing (D-037)". Story *playback* machinery exists (`storyEntered`/`storyStepped`/`storyExited`, `app/src/state/actions.ts:72-82`) but no authoring surface was found anywhere in `app/src` (grep for `P4.1`/`story.?auth` in `app/src`: no matches, this session) - playback and authoring are distinct capabilities and only the former exists. |
| P1.3 benchmark | Deferred to September (HANDOFF.md:286) | `not_started` | Matches claim; no contradicting evidence found. (ADR-004's own renderer-choice spike, ADR-004:10-27, is a related but distinct benchmark from candidate rendering approaches, not the P1.3 item.) |

### Verdict tally

| verdict | count | which |
|---|---|---|
| `verified_done` | 3 | drift #1, #5; remaining-work "relay steps 1-11" |
| `claimed_not_verified` | 1 | drift #2 (D-089 test tallies) |
| `partial` | 11 | drift #3, #4, #6, #7, #8, #9, #10, #11, #12, #13; remaining-work "snapshot pipeline" |
| `not_started` | 5 | remaining-work: pre-deploy gate, R2-2, recorded limitations, P4.1, P1.3 benchmark |
| `verified_externally_only` | 0 | none in the taxonomy-scored rows (the deploy-bar and D-089-deferred tables use their own director-specified columns, not this taxonomy, and are not included in this tally) |
| `structurally_untestable` | 0 | (keygen ceremony execution is the closest fit but is scored in the deploy-bar table's own columns, not tallied here, to avoid double-counting against a different rubric) |

Total 20 (13 drift rows + 7 remaining-work rows). Split by status as originally claimed:
13 were framed as accuracy-of-an-assertion claims (drift table), 7 as open work items
(remaining-work table).

**Headline number, scoped to what this reconciliation actually examined:** of the 13
HANDOFF.md/PLAN_1.0.md "Current Position" claims checked this session, **2 are cleanly
verified_done in-repo (~15%)**; 1 is claimed_not_verified; the other 10 are partial -
right in substance but carrying at least one stale number, omission, or superseded
detail. This number covers only the claims this bootstrap-mode reconciliation examined;
it is not a claim about HANDOFF's much larger "State of play" DONE list as a whole (see
"What is unverified," item 1).

---

## 6. The done-claims that do not hold, individually

**D-089's "check-all 12/12, relay 47/47, form 47/47"** (HANDOFF.md:273, DECISIONS.md:2291).
What it claims: all three suites green as of 2026-08-11. What this session holds: its
own check-all run is 11/12 (toolchain-drift clippy failure, tree otherwise unchanged),
and zero evidence either way on relay/form (Finding 2). What would settle it: running
`npm test` in `relay/` and `form/` on the current tree, and pinning a `rust-toolchain.toml`
to reproduce the clippy lint version at acceptance time. Probably true as of 2026-08-11;
unverified as of 2026-09-12.

**PLAN_1.0.md's "`app/src/ui/` is a README only"** (PLAN_1.0.md:82). What it claims:
no UI implementation exists. What the tree holds: 1,261 lines across 8+ files plus
`forms/` and `intake/` subdirectories, corroborated by MANIFEST.md's own later
description. What would settle it: nothing further needed - PLAN_1.0.md already
disclaims this inventory as unaudited (PLAN_1.0.md:121-123). Not a live claim so much
as an un-retracted stale one.

**PLAN_1.0.md's "`schemas/` contains only" two files** (PLAN_1.0.md:98). What it claims:
2 schema files. What the tree holds: 5. What would settle it: same self-disclaimer as
above: this whole "Absent" block is the 2026-07-11 snapshot, not re-audited since.

**PLAN_1.0.md's "cn-ingest ... placeholder library files"** (PLAN_1.0.md:95-96). Half
holds (cn-sync), half does not (cn-ingest has 10 modules and 76 tests). Probably true
when the 2026-07-11 snapshot was written; false today; same disclaimer applies.

---

## 7. Orphan files, ranked by in-degree

Not applicable in bootstrap mode. This requires a `TRACE.yaml` import-edge graph
(`project-trace`, not yet run) to rank files by importer count; without it, any list
here would be guessed rather than measured, which this reconciliation withholds rather
than fabricates.

---

## 8. Proposed dependency order for the open tasks

Not applicable in bootstrap mode, for the same reason as section 7: ordering by real
import edges needs the trace's file attachments, which do not exist yet. The 7 items in
the "v0.1.0 remaining work" table above are left unordered rather than sequenced by
guesswork.

---

## 9. Acceptance sentences that cannot be verified as written

A light pass only (this reconciliation is not a full acceptance-sentence audit of the
whole corpus - that sweep is not among the director's four required deliverables and
was not attempted at the same depth).

**C. Subjective or unenumerated predicates**

| Claim | Problem | Rewrite (same scope) |
|---|---|---|
| "the recorded ATNI Climate collective checkpoint" gates August pilots (HANDOFF.md:28-29, D-059.9; CLAUDE.md non-negotiables) | No schema or file format for what constitutes "recorded" was found anywhere in `docs/` or `schemas/`; the predicate ("recorded") is never enumerated - it could be a DECISIONS.md entry, a dated note, or something else, and nothing in-repo says which | "August pilots are conditional until a `D-059.9` entry is added to `DECISIONS.md` naming the checkpoint date and attendees" |
| D-023 "human review is recorded" (docs/design/intake-consent-text-draft-2026-07-24.md:3-7) | Same predicate, same gap: "recorded" names no artifact | "the form text ships once a `D-023.x` entry in `DECISIONS.md` states the sign-off explicitly, the way D-089 states its own disposition" |

No self-satisfying disjunctions (class A) or process-rules-dressed-as-state-assertions
(class B) were found in the claims this reconciliation actually examined; that absence
is reported rather than assumed, since this was a light, non-exhaustive pass (see
"What is unverified," item 9).

---

## 10. Proposed diff to HANDOFF.md / PLAN_1.0.md - NOT APPLIED

Omitted. The director's brief for this run states plainly: "NO RECOMMENDATIONS anywhere
in this file - it judges, it does not advise." A proposed diff is a recommendation for
how to edit the authority documents, so none is offered here. The findings above stand
as the judgment; what to do about them is left to the owner.

---

## What is unverified

1. **The headline number in section 5 is scoped, not universal.** It covers only the
   13 drift-table claims this reconciliation chose to examine (the ones the director
   named plus the ones found while checking them), not HANDOFF's full "State of play"
   DONE list, which runs to 15+ further bullets (Phases 0-2, Phase 3 facilitator role,
   the R2 EntityDetail fix, the D-055/D-058/D-059/D-060 sweeps, blueprint steps 1-10
   individually) that neither the scouts nor this session re-verified line by line.
2. **relay/ and form/ test results (47/47 each per D-089) are claims, not checks, this
   session** - see Finding 2. Nobody ran `npm test` in either directory this session.
3. **The keygen ceremony's actual execution is attested nowhere this reconciliation can
   see, by design** - the code exists and is correctly offline-only, but whether a real
   ceremony has been run for a real pilot key is a fact that lives entirely off-repo
   (deploy-bar table, `structurally_untestable` in spirit though not tallied under that
   label to avoid double-counting against the deploy-bar table's own rubric).
4. **The `verified_done`/`partial`/`not_started` labels applied to inventory and state
   claims (not task-completion claims) are a stretched use of a six-term taxonomy built
   for task verdicts** - named explicitly in the methodology note above section 5A, not
   silently assumed.
5. **`docs/PROJECT_PLAN.md` section 3 milestones, full `DECISIONS.md` text outside the
   sampled entries (D-023, D-080..D-089), `scripts/hooks/*` contents, `.codex/`
   contents (40+ files, existence-only checked), `app/index.html`, and line counts for
   `app/src/ui/forms/` and `app/src/ui/intake/`** were named by name in scout reports as
   not inspected and remain not inspected by this session either.
6. **Core CLI (`core/cli/src/`, `core/cli/tests/`) test counts** were not obtained -
   scout-core hit its tool-call cap before counting them; only the 9 library crates'
   197 tests are counted in this reconciliation's evidence.
7. **No scout or this session ran the relay or form test suites, the
   `scripts/e2e-remote-intake.ps1` script, or any browser this session** - stated
   directly per the director's brief, and this is the single largest gap behind the
   deploy-bar table's three "Yes" blockers.
8. **Heuristic named as a heuristic:** "the later-timestamped or later-refreshed
   document wins" was used once, informally, to resolve the three-way commit-count
   figure (Arithmetic note) - a recency judgment by document self-declared date, not by
   git history of the documents themselves (no `git log -1 -- <path>` was run per
   planning file, since this bootstrap run has no per-file staleness sweep to perform).
9. **The acceptance-sentence pass (section 9) is light and non-exhaustive** - two
   subjective-predicate examples were found and reported; classes A and B returned no
   examples in what was actually reviewed, which is not the same claim as "none exist
   anywhere in the corpus."
10. **Sections 7 and 8 (orphan files, dependency order) are withheld entirely**, not
    populated with placeholders, because they require a trace this repo does not yet
    have. Running `project-trace` first, as the skill's own chain specifies, would
    remove this gap for the next reconciliation.
