# HANDOFF.md - Live State (pickup target)

> The /pickup target; outranks session memory. Reading order for a fresh session:
> `CLAUDE.md`, this file, `docs/NEXT_SESSION.md` (launch cards), then
> `docs/planning/RELATIVES-ACROSS-GRAPHS-2026-11-11.md` (goals, state, possibilities, and
> the recommended sequence of record), then `docs/adr/ADR-008-two-sided-relations.md` and
> `docs/adr/ADR-007-network-circle-and-bridging.md` (both Proposed).
> Written 2026-11-11 at the close of the "relatives across graphs" exploration session
> (D-107). Previous handoff: `docs/archive/handoffs/2026-09-14-convention-sprint.md`.
>
> **The repo is PUBLIC** (origin `https://github.com/atniclimate/community-connector`).
> Plain `git push` fails from a non-interactive session here: `gh` has two accounts and the
> active one is not `atniclimate`. Working one-shot form (changes no config, prints no token):
> `git -c credential.helper= -c 'credential.helper=!f() { echo username=atniclimate; echo "password=$(gh auth token --user atniclimate)"; }; f' push origin main`.
> The 2026-11-12 trueup pushed main (the 2026-11-11 session's commits) to origin/main
> with the one-shot form above; the owner's eight uncommitted policy edits stay local.

## What this project is

Community Navigator (repo folder `community-connector`) is a local-first, privacy-first
tool for a community to see itself as a permission-filtered 3D graph of people,
committees, organizations, places, skills, and needs, and to route a need to people who
can meet it. A Rust core compiled to WASM owns data, schema, permissions, and graph
queries; the TypeScript app renders only the viewer-scoped projections it is handed.

The primary goal, named on 2026-11-11 (D-107): a web of relatives that can find each other
across institutions, in which each person is the center of their own view, each entity
shares as little or as much as it chooses, and consenting entities bridge between graphs.
TSDF T1 ("relationships are mutual and ongoing, not transactional") is that destination's
definition. The convention work (A9 constellation, facilitator intake, sealed QR form) is
the on-ramp. Everything runs on synthetic data.

## Where everything lives

| What | Path |
|---|---|
| Durable contract (mission, R1-R10, gates, autonomy) | `CLAUDE.md` |
| Invariants I1-I12 (review standard) | `AGENTS.md` |
| Decision register (D-001..D-107 with addenda) | `DECISIONS.md` |
| Vision, state, possibilities, and sequence of record (2026-11-11) | `docs/planning/RELATIVES-ACROSS-GRAPHS-2026-11-11.md` |
| Evidence register for the vision (R1-R8 findings, Jev checks, corrections) | `docs/research/relatives-discovery-2026-11-11.md` |
| Proposed ADRs: two-sided relations; Network circle and bridging | `docs/adr/ADR-008-two-sided-relations.md`, `docs/adr/ADR-007-network-circle-and-bridging.md` |
| Convention sprint plan (historical; parked one-liners in section 4) | `docs/planning/CONVENTION-SPRINT-2026-09-14.md` |
| Post-convention plan (S-E1..S-E5, go-live checklist) | `docs/planning/NEXT-PHASE-convention-e2e.md` |
| Session plan (R-sessions, historical) | `SESSION_ROSTER.yaml` |
| Route map to 1.0 and task trace (boxes unticked, not a status source) | `PLAN_1.0.md`, `TRACE.yaml` |
| Reveal launcher, key table, A9 cue sheet, presenter beats | `scripts/reveal.ps1`, `CONTROLS.md`, `app/public/beats.atni.json` |
| Relatives-demo synthetic group (dev viewer `?group=relatives-demo`) | `fixtures/templates/relatives-demo.template.json`, `fixtures/groups/relatives-demo.ops.jsonl`, `app/scripts/generate-relatives-ops.mjs`, `fixtures/README.md` |
| ATNI convention synthetic template, fixture, generator, edge checker | `fixtures/templates/atni-convention.template.json`, `fixtures/groups/atni-convention.ops.jsonl`, `app/scripts/generate-atni-ops.mjs`, `app/scripts/check-atni-edges.mjs` |
| Record mirror and analysis scope (cn-perm) | `core/crates/cn-perm/src/mirror.rs`, `core/crates/cn-perm/src/viewer.rs` |
| Continuance and introductions (cn-graph), their cn-api handlers | `core/crates/cn-graph/src/{structure,introductions,index,measures,query}.rs`, `core/crates/cn-api/src/analysis.rs` |
| Relatives panel, app analysis state, ring layout | `app/src/ui/relatives.ts`, `app/src/state/analysis.ts`, `app/src/viz/layout.ts` (`computeEgoLayout`), `app/src/viz/layout-mode.ts` |
| ATNI design system digest (source stays off-repo at `I:\ATNI_design-system\`) | `docs/design/atni-design-system-digest.md` |
| Verification battery (12 members) and pre-commit hook | `scripts/check-all.ps1`, `scripts/hooks/pre-commit` |
| Rust core and `cn` CLI | `core/crates/cn-{model,schema,store,perm,graph,api,wasm,ingest,sync}`, `core/cli` |
| Remote intake (built, accepted D-089, not deployed) | `form/`, `relay/`, `core/cli/src/intake/` |
| App (renderer `viz/`, presenter `viz/presenter.ts`) | `app/src/{viz,ui,state,wasm,theme}` |
| S-E2 intake-to-edges, BRANCH ONLY, verdict owed by the human | branch `s-e2-intake-edges` at `6c92eed`; its worktree folder is gone (`git worktree list` marks it prunable); the branch exists only in this clone |
| This session's lane branches (fully on main, keep until the owner removes them) | `relatives/perm` (`60a0fac`), `relatives/graph` (`4912f1a`), worktrees under `I:\claude-temp\claude\a6d7bcc1b798a7e3\5c5abc67-62a6-438f-8101-7db8887c8447\scratchpad\wt-{perm,graph}` |
| Codex review lane (off-repo) | `C:\dev\_reviews\community-connector\` |

## State of play

**DONE this session (2026-11-11, D-107; commits `f07e1bc..130e20d` plus the close docs):**
- Discovery: a 24-agent workflow (audits, eight research topics, six ideation lenses,
  three judges, Jev checks of 40 load-bearing sources) wrote the evidence register and
  the vision document.
- Defect fixes: foreign group id edges quarantine (`49a51cb`); edge submits no longer
  reveal hidden entities (`23aacec`, an EdgeCreate authorization rule change, see below);
  the permission spec matches the code (`3ce9823`); weighted routing with a hop limit
  (`f68b38e`) after an I5 split of `query.rs` (`0436d19`); hidden-entity invariance tests
  for every measure and query (`4aa654a`).
- First person-centered slice: relatives-demo fixture (`9777031`), record mirror
  (`c75f904`), analysis scope (`f0e76a6`), continuance (`5be76c8`), introductions
  (`0048e77`), cn-api and wasm exposure (`7c45afa`), app state (`8efb181`), Relatives
  panel (`8f8e203`), "My relatives" ring layout (`130e20d`). Routing (R6) is reachable
  from the app for the first time.
- The TSDF consent link on both consent paths (`ef3ee77`, `6db3c24`); this was NOT done
  item 0 of the previous handoff. Digest text unchanged.
- ADR-007 (`118f1be`) and ADR-008 (`b151878`) drafted, reviewed through three adversarial
  lenses plus Jev, every blocking finding applied; both Proposed, neither accepted.
- Verification: `pwsh scripts/check-all.ps1` at `130e20d` = 12/12 PASS (Rust 317 tests at
  the core merge; snapshot 0.85MB of 5MB; PII scan clean on 401 files); form suite 81/81
  and relay suite 47/47 outside check-all (the D-089-era "not re-run since 2026-08-11"
  warning is closed). The run used the owner's uncommitted script and hook edits, so it
  shows HEAD plus those edits.
- Try it: from `app/`, `npm run dev`, open with `?group=relatives-demo` (viewer is member
  `...127699`); search "eelgrass" under Who can help; press My relatives.

**Live warning, behavior change (`23aacec`):** members and facilitators are now denied
`target_missing` when they link to any entity outside their projection, including archived
or retracted entities. Governance keeps quarantine. The S-E2 branch and any intake or app
flow that links to such entities must be checked against this before merge.

**Before this session (unchanged):** the convention sprint of 2026-09-14 (A9
constellation, eight beats, ATNI stage styling, QR form on template 0.1.2 with the nine
questions and consent v3, D-103..D-106); relay steps 1-11 accepted (D-089). See the
archived handoff. **The convention outcome is unrecorded:** nothing in the repo says how
the A9 reveal went on 2026-09-15, and the D-052 retrospective was never held.

**NOT done - ordered next actions:**
0. **Walkthrough, demo, then decisions (owner's direction at the 2026-11-12 trueup):**
   walk the human through the 2026-11-11 changes, demo `?group=relatives-demo` in a real
   browser (first real-browser run of the Relatives panel), then record the human's
   answers to the one-line questions below in DECISIONS.md. Launch card:
   `docs/NEXT_SESSION.md`.
1. **ADR-008 acceptance, then its implementation** before the S-E2 branch or any form
   answer creates person-to-person edges: `EdgeAcknowledge`, Confirmed status, final
   private refusal, grandfathering in synthetic groups, template `subject` field, model
   0.1.2. The human accepts the ADR; building on synthetic fixtures is director work.
2. **In the same slice: an owner claim and a key-bound viewer.** Approved remote
   respondents land unowned (ADR-005 D7), so no person-centered feature reaches them;
   viewers are still asserted by the caller (D-015).
3. **ADR-007 Phase A** after its acceptance and after open question 5 (TSDF T2 mapping
   and reviewed T2 wording) is answered: injected time and declared use, agreements
   naming outside persons, bridge consent; receipts are their own later slice (D9).
4. **Verified logs and collective consent:** signed, hash-linked ops verified before fold
   (loaded logs fold without authorization today), a k-of-n governance seal for widening
   and ratification, trust grants that go inert when the grantor leaves.
5. **The StoryCreate existence oracle** (story steps naming a hidden entity quarantine
   `MissingTarget`): same fix pattern as `23aacec`.
6. **Human-only I9 walkthrough** of the Relatives panel, ring layout toggle, and TSDF link
   on both consent paths: keyboard order, 375px, screen reader, reduced motion. Nobody
   has seen them in a browser.
7. **UI copy review** (D-051, D-023) of all panel and toggle wording before anyone outside
   the team sees it.
8. **S-E2 verdict and merge** (D-056.1/2), human verdict first: re-check the branch
   against the EdgeCreate rule change and ADR-008; expect one import-line conflict in
   `app/src/ui/intake/panel.ts` (union); regenerate the ATNI fixture so the embedded
   template carries 0.1.2 (on main the QR form built from 0.1.2 cannot complete apply
   against the 0.1.1 fixture).
9. **Convention tail and deploy bar** (D-059.8 unmet): keygen ceremony, D-023 sign-off of
   the whole form page, the D-089 real-browser smoke, a Pages workflow, a guard against
   shipping the localhost relay origin, placeholder ids in `relay/wrangler.toml`, a named
   owner for the promised post-conference deletion (D-106, D-059.11).
10. Smaller follow-ups from the verifiers (vision doc section 8): routing fails whole on
    one failed path call; viewer center is the first owned entity; camera not refit on a
    layout switch; `willing_to_introduce` is case-sensitive; mirror's audience mapping
    duplicates `admissible_circle`; `fixtures/README.md` vocabulary sentence (D-044.4);
    I5 overages in `cn-api/src/lib.rs`, `app/src/viz/index.ts`, `cn-store/src/fold.rs`;
    the convention-era deferred advisories (beat-sheet `schema_version`, wiring-level
    tests). Optional cleanup: `git worktree prune` for the gone S-E2 folder (the branch
    stays).
- Known unverified: no browser or real-GPU check of anything built this session; the
  night sky has not been seen on a real GPU since the sprint.

## The human's queue

One line each; the vision document section 11 and the ADR open-question lists hold the
full text.

1. How did the A9 reveal land on 2026-09-15, and should the D-052 retrospective be
   recorded now?
2. Accept ADR-008 (two-sided relations) so it can be built first, as recommended?
3. Accept ADR-007 as a whole, and answer its question 5: the TSDF T2 mapping and the
   reviewed T2 wording that replaces "Shared within this group only" (Phase A
   precondition)?
4. Network compacts: is membership alone enough, or must a party share to see
   (contribution-gated reciprocity)? (ADR-007 Q1)
5. Who ratifies for a group: two governance members, or a k-of-n quorum per group?
   (ADR-007 Q2)
6. May governance audit which records crossed and who consented, and may it see
   refusals (never, counts only, or in full with a reason)? To answer with ATNI Climate
   (ADR-007 Q10 with ADR-008 Q1).
7. Default quiet period after a refusal: no end, one season, or a fixed term? (ADR-008 Q2)
8. Does an organization endpoint need its governance's acknowledgment? May an asker
   withdraw a pending request? (ADR-008 Q3, Q4)
9. Should facilitators (network weavers) see the full continuance picture, or only
   governance? (Facilitators get own records today, `f0e76a6`.)
10. May the S-E2 branch be backed up off this machine (it exists only in this clone)?
11. Which Nation, if any, would co-author a more-than-human relatives template?
12. Remaining ADR-007 one-liners (Q3, Q4, Q6-Q9, Q11-Q13): persons-only Phase A, default
    maximum duration, facilitators and agreements, T1 after Phase B, waiving acceptance,
    Network visibility as the person key, who attests network membership, small-cell
    threshold, co-produced data.
13. Carried from 2026-09-14: the parked convention one-liners (sprint plan section 4) and
    the S-E2 verdict; apply or drop the spine reconciliation note in the convention
    folder; the public product name on the form ("Community Connector" vs
    "Community Navigator"); consent v3 flags (a) "D-023 signed" for the whole form page,
    (b) keep or reword "no outside platforms, no third-party services", (c) who executes
    and records the promised deletion (D-059.11 purge sweep). The design-system authority
    question is closed (D-104 addendum: the 09/12 revision).
14. Go live or not (D-090c), and record, amend, or reject D-090c..D-095c. Standing:
    G-BACKUP accepted not solved.
15. The owner's uncommitted 2026-10-19 cadence edits (`AGENTS.md`, `CLAUDE.md`,
    `docs/LAUNCH_PROMPT.md`, the sprint plan, `scripts/check-all.ps1`, the hook and
    PII-scan scripts) have no DECISIONS entry and are the owner's to land.

## Non-negotiables a fresh session must not violate

- **Public repo, privacy first (I1):** no real-person PII in any file, commit, fixture, or
  Codex prompt; never commit `_private/`. Never copy ATNI logos, font files, or the
  design-system source into the repo; fonts ship via pinned open-licensed packages
  (D-101). On the projector, counts, never names (D-099); the render path enforces it in
  present mode.
- **The 2026-11-11 guard relaxation was for exploration only (D-107).** It does not carry
  forward: no real data, no gate crossed, ADR-007 and ADR-008 stay Proposed until the
  human accepts them, and ADR-007 Phases B and C are human gates.
- **Deploy bar D-059.8 is UNMET:** nothing goes live on Pages or Workers. No real ingestion
  before the recorded committee checkpoint. Cloudflare spend only for the relay, tier is a
  human call.
- **The graph never listens (ADR-005):** new entries reach the app only by a facilitator
  action (apply, then reload).
- Results that name keystone people are governance-only and unexportable; a person sees
  what rests on them (analysis scope, `f0e76a6`). Edge lists are never exported in bulk.
- Permission logic only in `cn-perm` (I2); app state mutates only through `app/src/state`
  (I4); provenance + tier on everything (I6); versioned formats (I7); snapshot under 5MB
  (I8); hyphens in docs (I10); reduced-motion variant for every motion (I9).
- ADR-004 renderer stands. Standing rulings no session may "fix": D-032, D-037, D-051,
  D-056.4. The S-E2 branch is never merged without the human's recorded verdict, and
  never touched or deleted by an agent.
- Never stage or commit the owner's dirty files; stage only your own paths and pass an
  explicit pathspec to `git commit`. Atomic conventional commits; no `--no-verify`.

## Key design commitments (shortest refresher)

- Rust/WASM core is the single source of truth; the app renders permission-filtered
  projections only; the op log is event-sourced and state is a fold over it.
- Hidden equals absent (ADR-003 A-B4): every measure, query, and write-path answer for a
  hidden id matches an id that never existed (`4aa654a`, `23aacec`).
- Relatives, not contacts: ties two-sided (ADR-008), introductions through willing
  relatives, continuance phrased as an invitation to share the load, bridging as
  TSDF-shaped, expiring, use-scoped agreements (ADR-007).
- Intake: in-app facilitator entry is primary; the remote QR path is built and accepted
  but not deployed; `cn intake apply` is the only writer.
- Presenter mode: beats from `app/public/beats.atni.json`, validated at load; `labelKinds`
  gates labels; `camera` controls motion; Escape exits; `r` toggles the rail.
- TSDF tier codes primary in the UI (D-032); in-app story authoring in v0.1 (D-037).
