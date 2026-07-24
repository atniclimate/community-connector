# community-connector - Plan to 1.0

**v1.1 - Phase 5 reconciliation (gate-grill rulings D-050..D-056, 2026-07-24)**

> Product name: Community Navigator (repo folder `community-connector`, per
> DECISIONS D-001). All work is governed by AGENTS.md I1-I12 (the canonical review
> standard); this plan does not restate the invariants, it links to them and names
> only the requirement-specific acceptance criteria that ride on a given invariant.
> Docs use hyphens, never em dashes. This document is a route map, not an accepted
> ADR: it overrides no accepted ADR and does not outrank the live HANDOFF.md. It was
> rewritten in Phase 3 (refinement) from the v0.1 draft, the project's own docs
> (CLAUDE.md, docs/PROJECT_PLAN.md section 3, HANDOFF.md,
> docs/design/integration-plan-2026-07-06.md, DECISIONS.md, the ADRs), and the Codex
> adversarial review; it was then expanded in Phase 4 (2026-07-11). Every contested
> claim, file path, module name, and count below was re-verified against the working
> tree on 2026-07-11; anything that had drifted was corrected in place. A fifth,
> lighter pass (Phase 5 reconciliation, 2026-07-24) reconciled the decision gates,
> remote/publish state, and sequencing with the gate-grill and look-back rulings
> D-050..D-056; it did not re-inventory code claims, which stay dated 2026-07-11. See
> ## Provenance for the full pipeline.

## 1.0 Definition

The project has one **committed near-term deliverable** and one **inferred** longer
line. Keep them separate; the review's central finding was that the v0.1 draft made an
inference operative.

- **v0.1.0 = "convention-pilot ready" (committed).** docs/PROJECT_PLAN.md section 3
  (D-039) defines it: a facilitator-run, snapshot-first build that ingests intake-form
  data, renders the participant and committee graph, routes needs to solutions, and
  presents authored stories at an ATNI Climate convention. Personal mode, identity,
  deep accessibility, and governance/tiering tooling are staged to v0.2. This is the
  real target of Phases 0-5 below.

- **1.0 = the full reusable R1-R10 tool (inferred, and flagged as inference).** No
  source document fixes a "1.0" boundary. docs/PROJECT_PLAN.md stages v0.1.0 then v0.2;
  it never names 1.0. So the 1.0 scope described here (Phases 6-9) is an inference about
  where Community Navigator becomes a tool a second community can adopt without a code
  fork or a resident developer. **G-RAT was RESOLVED 2026-07-24 (D-052):** v0.1.0 is
  ratified as the committed finish line for the convention arc, with a sharpened
  acceptance bar - ready for actual usage across the real entity kinds (persons,
  organizations, places, skills, needs) at pilot scale (~150 expected, 300 max
  signups), not demo-grade. Ratification of the fuller 1.0 line (Phases 6-9) is
  DEFERRED to a post-convention retrospective, when pilot evidence exists; until then
  Phases 6-9 remain deferred scenarios, not scheduled commitments.

For its author to call the project 1.0, a ratified 1.0 line would need R1-R10 hardened
into a release (reusable without a fork or resident dev; the full data-in surface via
`cn-ingest` + the `cn` CLI; personal mode with per-attribute sharing; network-readiness
and identity as validated abstractions; a fast permission-aware graph core re-measured
in-browser; stories, both distribution modes, and the R9 accessibility baseline; and
end-to-end exercise through a real pilot), plus at least the first sibling-tool
integration (cap-assessor) demonstrated through the shared importer and vocabulary. All
of that stays deferred pending the post-convention retrospective (D-052).

## Current Position

**Do not read "Phase 3 ~half done" as a feature-count.** The inherited "half" label
(HANDOFF.md, docs/PROJECT_PLAN.md) is a historical planning judgment. The review
re-inventoried the working tree; the honest picture is an acceptance-unit inventory, not
a percentage.

**Complete and accepted (verified 2026-07-11):**
- Architecture foundations: ADR-001..004 accepted through adversarial rounds
  (docs/adr/, with round-1/round-2 critique records committed).
- Rust/WASM core: `cn-model`, `cn-schema`, `cn-store`, `cn-perm`, `cn-graph`, `cn-api`,
  `cn-wasm` implemented (9 crates plus the `cn` CLI = 10 workspace members in
  `core/Cargo.toml`); constructors require provenance and sensitivity tier (I6);
  permission logic isolated in `cn-perm` (I2); `cn-api` exposes viewer-scoped
  projection, detail, search, path, neighborhood, report, submit, load, export.
  Blueprint tests across all six domain crates plus a measured API test
  (`core/crates/*/tests/blueprint.rs`, `core/crates/cn-api/tests/measure.rs`).
- Base renderer: instanced Three.js layer (ADR-004, measured on the reference Iris Xe)
  landed and director-accepted. App runs the synthetic fixture through a Web Worker,
  mutates state only through the explicit state machine (I4), honors
  `prefers-reduced-motion`, and mounts the theming pipeline. Viz modules present:
  `app/src/viz/{scene,layout,nodes,edges,halos,picking,camera,projection,quality,
  config,index}.ts`.

**Absent (verified against the tree, not inferred):**
- User workflows: labels, motion/focus polish, search/explore surfaces, detail and
  legend panels. `app/src/ui/` is a README only.
- The flat reading projection, the comprehension primer, and any authored/seeded
  stories.
- **Snapshot data payload.** `app/src/main.ts` gates the only fixture-load path behind
  `import.meta.env.DEV` (line 63, `loadDevDemo()` at line 70); `app/vite.config.ts`
  snapshot mode adds only `viteSingleFile()` (line 23) with no data-embedding step;
  `scripts/check-size.mjs` proves only that `dist/index.html` is under 5MB. A snapshot
  build today would ship an empty viewer that passes the size gate. This is a real gap,
  addressed in Phase 2.
- **Facilitator authority.** `GroupRole` (`core/crates/cn-model/src/group.rs`, lines
  55-60) has only `Member` and `Governance`; there is no `Facilitator`, and `cn-perm`
  exposes only `is_governance` (`cn-perm/src/viewer.rs`). Entry/mutation authority
  (D-028) does not yet exist.
- Ingest and CLI: `core/crates/cn-ingest/src/lib.rs` and `core/crates/cn-sync/src/lib.rs`
  are placeholder library files; the `cn` CLI (`core/cli/src/main.rs`) prints a scaffold
  message. No importer, validate, export, or snapshot command.
- **Persisted-format schemas.** `schemas/` contains only `group-template.schema.json`
  and `theme-tokens.schema.json`; `schemas/README.md` names ingest, Codex-output, and
  story-path schemas, and AGENTS.md I7 additionally requires op-log and export schemas -
  none of these exist. Story save/reload, ingest, export, and the snapshot envelope all
  need versioned schemas with unknown-major rejection (I7) before they are built.
- Everything in v0.2+ and the inferred 1.0 line.

**Guardrails and standing state:** the PII scan (`scripts/pii-scan.ps1`) and the 5MB
snapshot budget check (`scripts/check-size.mjs`) exist. Two contrasting synthetic group
templates exist (`fixtures/templates/research-network.template.json`,
`fixtures/templates/fisheries-committee.template.json`). As of 2026-07-11 the git tree
was a single local `main` with no remotes, 67 commits, last commit 2026-07-06
(`856ccf8`, docs: THE_STORY), with MANIFEST.md and PLAN_1.0.md both untracked.
**Updated 2026-07-24:** the human created the public remote
`atniclimate/community-connector` (D-053); the remote, hosting-vendor, and spend gates
are conditionally OPENED for exactly that one path (the public remote plus the
Cloudflare Workers intake relay), with push/deploy preconditions: license in-repo
(satisfied, D-054), the D-055 pre-publish sweep passed, and core stability. All other
gates stand unchanged. PLAN_1.0.md, MANIFEST.md, and DEPENDENCIES.md become tracked
after the D-055 sweep; this revision is part of that sweep. Open human gates now: the
recorded ATNI Climate collective checkpoint before the FIRST August internal-pilot
ingestion (D-030/D-050), community-facing text review (D-023), G2 Open Eligibility
licensing, and G-BACKUP. G-RAT, G-DATE, G1, Q-B, Q-C, and P0.6 are RESOLVED
(D-050..D-055) - see DECISION GATES below. Live completion state lives in HANDOFF.md
"State of play"; the inventory above is the 2026-07-11 snapshot and was not re-audited
by this reconciliation.

## Working-tree congruence (organizational structures needed)

The review found the tree is not congruent with the plan's own claimed state. To reach
plan congruence (executed in Phase 0, decisions reserved to the human where noted):

- **Untracked planning docs.** RESOLVED 2026-07-24 (D-055): PLAN_1.0.md, MANIFEST.md,
  and DEPENDENCIES.md become tracked, contingent on the pre-publish world-readability
  sweep (machine paths, remediation references, convention logistics, D-023-pending
  text). Until the sweep unit runs, the three docs stay uncommitted; borderline content
  moves to a gitignored `_private/` and is reported.
- **Stale status docs / single source of truth.** At review time (2026-07-11),
  HANDOFF.md (then dated 2026-07-06) still framed plan-v3 as the preferred next
  sitting; MANIFEST.md described state as of 2026-07-11. HANDOFF.md has since been
  refreshed (2026-07-24) and remains the live authority. When a gate resolves, update DECISIONS.md,
  docs/PROJECT_PLAN.md, HANDOFF.md, and docs/NEXT_SESSION.md **as one atomic change**,
  rather than layering a new plan doc on stale ones (the 2026-07-24 true-up did this
  for the D-050..D-056 rulings; PROJECT_PLAN.md's full revision lands with the D-055
  sweep unit, per D-056.5).
- **Doc-vs-code drift.** `schemas/README.md` names schemas that do not exist; the "half
  done" label overstates feature completeness. Phase 0 records the acceptance-unit
  inventory as the single truth and Phase 2 closes the schema drift.
- **Branch discipline.** Preserve single-branch `main` unless the human deliberately
  adopts a branch/worktree policy; there is no evidence of one today.
- **Reorg deferral.** MANIFEST.md's directory-reorganization proposal (its "Proposed
  Reorganization (NOT EXECUTED)" section) is NOT executed and must not be conflated with
  plan ratification; defer it until after the pilot critical path to avoid reference
  churn during delivery.

## Plan Congruence

Standing rules that keep this plan, the manifest, and the repo's own status docs from
drifting. The governing principle: **each fact has exactly one source of truth, and this
plan is subordinate to all of them.**

- **Single source of truth map.**
  - **HANDOFF.md** - live state: phase status, open gates, degraded modes, next
    actions. Outranks session memory and this plan. If HANDOFF.md and PLAN_1.0.md
    disagree about current state, HANDOFF.md wins and this plan is corrected.
  - **DECISIONS.md** - the durable record of judgment calls, ladder climbs, adversarial
    outcomes, and gate resolutions (D-001..D-056 as of 2026-07-24). A gate is not
    "resolved" until it has a DECISIONS.md entry.
  - **docs/PROJECT_PLAN.md section 3** - Execution Plan v2 (D-039), the accepted
    session-by-session plan for v0.1.0. This plan's Phases 0-5 align to it; where they
    add detail (the integration-plan corrections), they cite it.
  - **AGENTS.md** - the I1-I12 invariants, the review standard for every diff. This plan
    links to it and never restates it.
  - **CLAUDE.md** - the durable contract (mission, requirements R1-R10, architecture
    stances, human gates, autonomy protocol).
  - **docs/adr/** - accepted architecture decisions. This plan overrides no accepted ADR;
    architecture changes go through an ADR plus one adversarial Codex round, not through
    an edit here.
  - **docs/design/integration-plan-2026-07-06.md** - PROVISIONAL historical input;
    changes no accepted ADR. Its routing/ingest specs, formerly parked on G-RAT, are
    now scheduled v0.1.0 work (D-052; docs/PROJECT_PLAN.md section 3 commits them).
  - **MANIFEST.md** - a dated inventory snapshot plus an unexecuted reorg proposal. It is
    descriptive, not governing; its reorg is never treated as ratified.
  - **PLAN_1.0.md** (this file) - the route map to 1.0. It sequences and anchors the
    work; it does not outrank any doc above. Its tracked status is decided (D-055,
    2026-07-24): tracked once the pre-publish sweep passes. Tracked or not, it remains
    a route map, never a governing doc.
- **When to update this plan.** After each phase exit (record actual acceptance units
  and the reforecast); when a decision gate resolves (mirror the resolution here and in
  DECISIONS.md in the same atomic change); and whenever the tree drifts from a claim here
  (correct the claim immediately - never let a stale path or count rot in this file).
- **Atomic doc-update rule.** A ratification or gate resolution that touches multiple
  authorities (for example G-RAT: DECISIONS.md, docs/PROJECT_PLAN.md, HANDOFF.md,
  docs/NEXT_SESSION.md, and this plan) lands as one commit, not a layered sequence.
- **Drift-check discipline.** Before repeating any file path, module name, doc title, or
  count from this plan in a session, re-verify it against the working tree. The Phase 4
  expansion did so on 2026-07-11; the Phase 5 reconciliation re-checked the gate,
  sequencing, remote/publish, and doc-status claims against DECISIONS.md and HANDOFF.md
  on 2026-07-24 (code-inventory claims were not re-verified and stay dated 2026-07-11);
  the next editor does the same and dates the check.

## Decision Gates

Each gate has a stated owner, a default (what proceeds if the owner is silent), and the
evidence the owner needs to decide. Gates are never crossed for momentum (CLAUDE.md
prime directive 2).

**File-resolvability note (Phase 4 re-check, 2026-07-11).** None of the six gates below
is resolvable by file evidence. Each turns on an external fact (a convention date), a
human authority decision (ratification, vocabulary, licensing, backup, network), or a
human review of community-facing text. This expansion re-checked the tree and confirms
all six remain genuinely human-owned; none moved into the plan body. **Update
2026-07-24:** the human answered the queued gates in the gate-grill session. G-RAT
(D-052), G-DATE (D-050), and G1 (D-051) are RESOLVED below, as are Q-B (D-053), Q-C
(D-054), and P0.6 (D-055). G2, G-BACKUP, and G-NET remain open.

### G-RAT - 1.0 line ratification (RESOLVED 2026-07-24, D-052)
- **Question (as posed).** Is the inferred 1.0 line (Phases 6-9) the right target, or is
  v0.1.0 the actual finish line with everything past it aspirational?
- **Resolution (D-052).** v0.1.0 is ratified as the committed finish line for the
  convention arc, with a sharpened acceptance bar: ready for ACTUAL USAGE across the
  real entity kinds (persons, organizations, places, skills, needs) at pilot scale
  (~150 expected, 300 max signups), not demo-grade. Ratification of the fuller 1.0 line
  (Phases 6-9) is DEFERRED to a post-convention retrospective, when pilot evidence
  exists.
- **Effect.** Phase 5's formerly G-RAT-parked routing and importer items (P5.1-P5.3)
  are now scheduled v0.1.0 work - docs/PROJECT_PLAN.md section 3 already commits them.
  Phases 6-9 stay deferred scenarios (not ratified INTO a 1.0 line) until the
  retrospective; every "G-RAT" dependency inside Phases 6-9 and the Milestones table
  now reads "post-convention retrospective ratification".

### G-DATE - Convention date and consent runway (RESOLVED 2026-07-24, D-050; Q-A)
- **Resolution (D-050).** The ATNI Annual Convention is 2026-09-14; early September is
  the soft deadline for pre-convention consent, and most joins are expected at the
  convention itself. New commitment: internal pilots with several trusted groups run in
  August, BEFORE the convention, on the NORMAL app build (the snapshot pipeline is a
  convention deliverable, not a pilot blocker; D-056.3).
- **Consequences.** The intake -> review -> ingest -> render pipeline must be usable by
  mid-to-late August; the D-030/D-034 consent process (form-based individual consent,
  outside-repo staging, T1 tiering) applies to the internal pilots too; the recorded
  ATNI Climate collective checkpoint must precede the FIRST August internal-pilot
  ingestion, not merely the convention.

### G1 - Vocabulary authority (RESOLVED 2026-07-24, D-051; D-041)
- **Resolution (D-051).** ATNI Climate authors the capability vocabulary in its own
  words, but language work is sequenced AFTER the system is fully functional and
  stable. Until then the backend/schema layer uses standard developer language over the
  empty HSDS-shaped structure (the D-044.4 mechanics/language split). A design pass
  over front-facing interface text and language corrections is planned for the later
  stage. No settler vocabulary is committed as ATNI's terms; community-facing text
  still requires D-023 human review before use.

### G2 - Open Eligibility licensing (MAJOR; D-041)
- **Owner.** The human, with counsel.
- **Default.** No Open Eligibility mapping in v0.1. Any later mapping is isolated as
  separately-licensed (CC BY-SA) third-party data with attribution and a NOTICE, kept out
  of the PolyForm-licensed code.
- **Evidence the owner needs.** A confirmed FHIR/interop need, plus counsel's read on the
  exact CC BY-SA version (the conflict holds for any version).

### Q-B - Intake / form platform (RESOLVED 2026-07-24, D-053)
- **Resolution (D-053).** NO external form platform (no Google, no Microsoft) - the
  form-platform question is replaced entirely. Direct in-app entry is an initial
  feature: P3.5/P3.6 ARE the intake pipeline, not UI polish. ALL submissions (in-app or
  remote) land in a facilitator pending-review queue and enter the graph only on
  facilitator approval. The remote path is QR -> static intake form on GitHub Pages ->
  client-side sealed-box encryption to the facilitator public key (private key only on
  the pilot PC) -> minimal Cloudflare Workers + KV ciphertext relay -> pilot-PC
  pull/decrypt/stage, then relay wipe. No auth in v0.1.0 (or v1.0). CSV ingestion
  becomes a secondary path for structured sources. Scoped by the ADR-005 draft
  (docs/adr/ADR-005-remote-intake.md, required per D-056.1).
- **Gates opened by the human (conditional).** The public remote
  `atniclimate/community-connector` (created 2026-07-24), Cloudflare Workers hosting on
  the human's account, and the associated spend - for this one path only.
  Push/deploy preconditions: license in-repo (satisfied, D-054), the D-055 sweep
  passed, core stability. The R5 "not networked" stance is amended for this one intake
  path only; a localized offline intake system is the later stretch goal.

### Q-C - License (RESOLVED 2026-07-24, D-054)
- **Resolution (D-054).** PolyForm Noncommercial 1.0.0, committed to the repo as
  LICENSE.md before the first push to the public remote (precondition satisfied on
  disk; re-verify at push time).

### P0.6 - Tracked status of the root planning docs (RESOLVED 2026-07-24, D-055)
- **Resolution (D-055).** PLAN_1.0.md, MANIFEST.md, and DEPENDENCIES.md become tracked
  authorities, contingent on the pre-publish world-readability sweep passing; until the
  sweep unit runs, the three docs stay uncommitted.

### G-BACKUP - Single-machine backup mitigation (MAJOR; D-026)
- **Owner.** The human (maintainer); re-raised every decision session.
- **Default.** Risk remains ACCEPTED; no remote, no bundles; the remotes gate stays
  closed; no autonomous action.
- **Interim mitigation offered (does NOT cross the no-remotes gate).** A maintainer-run
  local file mirror to a distinct physical H: drive would reduce single-disk failure risk
  without creating a git remote. It is a plausible interim measure only, and the decision
  stays with the human.
- **Evidence the owner needs.** Verify H: is a distinct physical device (not a partition
  of the same disk), encrypted, available, and suitable; accept that a local mirror is
  NOT equivalent to off-site/remote backup (it does not protect against theft, fire,
  malware, or mirrored accidental deletion/corruption). If approved, record a dated
  local-mirror procedure with restore verification and explicit exclusion of real-data
  staging in DECISIONS.md. Do not automate it without approval.
- **Note (2026-07-24).** The new public remote (D-053) will hold code only, never data,
  so it is not a backup answer for operational/pilot data; the risk remains ACCEPTED
  and open.

### G-NET - External protocol / identity / federation choice (standing; R5)
- **Owner.** The human; the abstraction is the maintainer's to build, the network choice
  is the human's.
- **Default.** Build the abstraction (`cn-sync` `SyncTransport` + local adapter); park
  any commitment to a specific protocol, identity standard, federation, or vendor.
- **Evidence the owner needs.** The Phase 7 identity ADR's framed options. Designing the
  abstraction is autonomous; choosing the network is the human's.
- **Note (2026-07-24).** D-053 amends the R5 "not networked" stance for exactly one
  intake path (the sealed-envelope relay), scoped by the ADR-005 draft as INGEST, not
  sync - the puller routes through cn-ingest concepts, never the `SyncTransport` seam,
  and the graph never listens on the network. The general
  protocol/identity/federation choice remains parked here.

### Gate-to-microtask blocking map

Updated 2026-07-24 for the D-050..D-056 resolutions. Nothing parks on
G-RAT/G-DATE/G1/Q-B/Q-C any more except what the rulings themselves defer (noted per
row); the open rows are G2, human review (D-023) plus the D-050 collective checkpoint,
and G-BACKUP.

| Gate | Parks until answered | Proceeds gate-blind in parallel |
|---|---|---|
| G-RAT (RESOLVED, D-052) | Nothing parks on G-RAT. P5.1-P5.3 are scheduled v0.1.0 work (PROJECT_PLAN section 3 commits them). Phases 6-9 are deferred BY THE RULING to the post-convention retrospective, not parked on an open gate | All v0.1.0 work |
| G-DATE (RESOLVED, D-050) | Nothing; the calendar is set (convention 2026-09-14, August internal pilots on the normal app build). Real ingestion still waits on the D-030/D-050 collective checkpoint and D-023 review | All synthetic-data build, test, and rehearsal work |
| G1 (RESOLVED, D-051) | Nothing parks; ATNI vocabulary authoring is sequenced BY THE RULING to post-stability. Standard developer language over the empty HSDS-shaped structure proceeds now | Everything else, including P5.5's structure and mechanics |
| G2 | Any Open Eligibility / FHIR mapping | All pilot vocabulary and routing (ATNI's own terms) |
| Q-B (RESOLVED, D-053) | Nothing; the intake path is decided (in-app entry + sealed-envelope relay, pending-review queue). Any PUSH/DEPLOY on the opened path waits on the D-053/D-055 preconditions (license in-repo, sweep passed, core stability) | Intake-pipeline build: P3.5/P3.6, relay work per the ADR-005 draft, CSV secondary path |
| Q-C (RESOLVED, D-054) | Nothing; PolyForm Noncommercial 1.0.0 is tracked | All development |
| Human review (D-023) | All community-facing text (now concretely the relay intake-form and consent text); any real ingestion (also gated by the D-050 collective checkpoint) | All synthetic-data build, test, and rehearsal work |
| G-BACKUP | Any backup automation; any local-mirror automation (the code-only public remote is not a backup answer) | All development (mitigation is optional and human-driven) |

## Risks and Mitigations

The top eight risks to reaching 1.0, spanning technical, budget, calendar, and
single-maintainer throughput. Each carries a mitigation already wired into the plan and
an early-warning signal to watch for.

1. **[RETIRED 2026-07-24, D-052] G-RAT never resolves, so the 1.0 line stays inferred
   and Phases 6-9 stay unschedulable (calendar/scope).** Resolved: v0.1.0 is ratified as
   the convention-arc finish line; Phases 6-9 defer to the post-convention
   retrospective. The residual risk is scope pressure inside the v0.1.0 window against
   the sharpened real-usage acceptance bar, carried by risks 2 and 3.

2. **[PARTIALLY RETIRED 2026-07-24, D-050] Convention-date runway is too short,
   compressing or breaking the pilot arc (calendar).** The date is set: convention
   2026-09-14, August internal pilots on the normal app build. The live calendar risk
   is now the mid-to-late-August intake-pipeline deadline, and D-056.4 names the least
   compressible items as human-path: the D-023 text review and the recorded ATNI
   Climate collective checkpoint before the FIRST August ingestion.
   - *Early warning.* The intake pipeline (P3.5/P3.6 + relay) is not usable by
     mid-August; the collective checkpoint or D-023 review has no date as August opens.

3. **Single-maintainer throughput and single point of failure (throughput).** Illness,
   usage-limit exhaustion, or lost session context stall a project with one director and
   no team.
   - *Mitigation.* Atomic per-unit commits; HANDOFF.md session-end discipline; the Codex
     usage-failover directive (`grind`/`review` profiles); the 9-12-session figure held as
     a capacity hypothesis with reforecasts at Phase 6/7/8 exits.
   - *Early warning.* A 2x estimate blowout on any phase; rising focused-hours per
     accepted acceptance unit; HANDOFF.md not refreshed at session end.

4. **Single-machine total data loss (G-BACKUP / D-026), technical/budget.** A disk
   failure loses everything not yet pushed; the 2026-07-24 public remote will hold code
   only, never data, so it is not a backup answer for operational/pilot material.
   - *Mitigation.* Risk explicitly ACCEPTED and re-raised every decision session; an
     optional maintainer-run H: local mirror is offered as interim mitigation that does
     not cross the no-remotes gate; re-raised as a release blocker at Phase 9 (P9.7).
   - *Early warning.* Disk SMART warnings or read errors; G-BACKUP still ACCEPTED with no
     mirror in place at a milestone boundary.

5. **Performance and size claims do not hold in-browser at pilot scale (technical).** The
   p95 70ms frame tail (ADR-004) versus the 50ms S3-A2 goal and the 2.9MB projection JSON
   per recompute (docs/PROJECT_PLAN.md risk register) are allocations, not in-browser
   measurements.
   - *Mitigation.* Re-measure with labels and halos live (P1.3 - deferred to September,
     post-pilot, per D-056.3) and at pilot-realistic scale in Phase 5 (P5.8); ADR-003
     D2's typed-array escape hatch and halo-mitigation are the named levers; the 5MB
     budget fails the build loudly, never degrades silently.
   - *Early warning.* Phase 1 benchmark tail exceeds 50ms with labels+halos; the snapshot
     size trends toward 5MB as real projection data lands.

6. **Enforcement gap: the AGENTS.md battery is a human checklist with no CI (technical).**
   Only the staged PII scan is enforced today, so regressions can land silently and later
   phases' "green" exit criteria would be unreliable.
   - *Mitigation.* Phase 0 is the gating precondition for everything downstream: P0.2
     builds the single `scripts/check-all` orchestrator and P0.3 enforces it via a local
     hook (no remote needed). No later exit criterion counts until this lands.
   - *Early warning.* Any phase reports exit "green" without `check-all` having run; the
     local hook is bypassed or absent.

7. **Community-facing text and vocabulary authority mishandled (sovereignty; G1, D-023,
   D-034).** A settler taxonomy or unreviewed wording would violate sovereignty or produce
   an unusable pilot.
   - *Mitigation.* G1 parks the capability terms (empty HSDS structure proceeds); the
     human-review gate (D-023) blocks all community-facing text until reviewed; ATNI
     Climate authors terms under FPIC.
   - *Early warning.* Any commit touches capability terms without a recorded human review;
     Open Eligibility or any external taxonomy appears in a fixture or template.

8. **Idempotent re-ingest is subtle and gets it wrong (technical).** Same-day QR joiners
   (D-030) require fast re-ingest; a flawed identity key would duplicate entities/edges at
   the convention.
   - *Mitigation.* Deterministic UUIDv5 identity plus a stable `source_row_id` key,
     `AttributeSet`-on-reimport, and a round-trip property test asserting zero new
     creates / zero duplicate edges / zero custody growth (P5.3); director blueprint plus
     a mandatory adversarial round because the semantics are subtle.
   - *Early warning.* The re-import property test shows any nonzero new-create, duplicate
     edge, or custody-event growth on identical input.

## Phased Plan

> Phases 0-5 reach the **committed v0.1.0 convention-pilot milestone** and align with
> Execution Plan v2 (docs/PROJECT_PLAN.md section 3) as corrected by the provisional
> integration plan. Phases 6-9 describe the **inferred 1.0 line** and are DEFERRED to
> the post-convention retrospective (D-052) - read every "G-RAT" dependency inside
> Phases 6-9 and the Milestones table as "post-convention retrospective ratification".
> Every phase preserves the operating rules: foundations get the deep-thinking ladder
> and a mandatory adversarial Codex round; permission-adjacent work is grind HIGH +
> adversarial; human gates are parked, never crossed.
>
> **Microtask anchors.** Each microtask carries a stable ID (`P<phase>.<n>`), the
> files/modules/docs it *Touches*, the command or check that *Verifies* it, and its
> *Depends* on other microtasks (by ID) or milestones. Where a verify says "`check-all`
> green," that is the Phase 0 orchestrator (P0.2); it is therefore an implicit dependency
> of every post-Phase-0 microtask and is not relisted under *Depends*.
>
> **Verification: automated vs human-only.** The review found the only ENFORCED gate
> today is the staged PII scan (`scripts/hooks/pre-commit`, `core.hooksPath =
> scripts/hooks`, both confirmed 2026-07-11); the full battery in AGENTS.md is a
> documented human checklist, not an enforced automated gate, and there is no CI.
> Therefore **no later phase's exit criteria are reliable until Phase 0 makes the battery
> a single reproducible command and enforces it.** Each phase names its verification
> mechanism: the Phase 0 orchestrator (`check-all`), phase-specific tests, a Codex review
> round, and any human-only gate (screenshot review, accessibility testing, rehearsal,
> consent/language review) called out explicitly because a machine cannot close it.

### Route as resequenced 2026-07-24 (D-056)

HANDOFF.md "State of play" is the live order of work and the live completion record;
the phase bodies below remain the durable task anchors. Per HANDOFF.md at this
reconciliation: Phase 0 is complete; Phase 1 is complete except P1.3; Phase 2's schemas
(P2.1/P2.2) landed; Phase 3's role work (P3.1-P3.4) landed with its adversarial round;
a Phase 5 slice (partial P5.4 CLI, P5.7 recipe, P5.10 evidence template) landed.
Phase-body kickoff/verify steps below that describe the pre-completion tree (no
remotes, untracked docs, missing `check-all`) are historical. For the August pilot
window - internal pilots run the NORMAL app build; the snapshot pipeline is a
convention deliverable, not a pilot blocker (D-056.3) - the order is:

1. **R2 EntityDetail fixes first** (reviewer-confirmed defects, D-049);
   permission-adjacent, so a fresh adversarial round follows the fix.
2. **Long-lead gate-openers in parallel** (D-056.3): the D-055 pre-publish sweep (this
   doc's update is part of it); the ADR-005 "Remote intake: sealed-envelope relay and
   facilitator pending-review queue" draft (docs/adr/ADR-005-remote-intake.md, required
   scope in D-056.1) plus its adversarial round; the intake-form/consent text drafted
   into D-023 human review; the facilitator keygen ceremony design (offline private-key
   backup, key-fingerprint pinning in the puller).
3. **P3.5 wizard + P3.6 entry forms - THE intake pipeline (D-053):** direct in-app
   entry plus the pending-review staging queue. Every submission (in-app or remote)
   lands pending and enters the graph only on facilitator approval; queue persistence
   is durable-write-first, gitignored staging, pii-scan covered, with near-duplicate
   surfacing (D-056.4). Approved remote entries land unowned, facilitator-created
   (D-056.2).
4. **Remote intake relay** (D-053, per the ADR-005 draft): static Pages intake form,
   client-side sealed-box encryption to the facilitator public key, minimal Cloudflare
   Workers + KV ciphertext relay (payload size caps, rate limits, KV TTL), pilot-PC
   puller that decrypts locally, stages durably into the review queue, THEN wipes the
   relay. Push/deploy only after the D-053/D-055 preconditions (license in-repo -
   satisfied; sweep passed; core stability).
5. **Snapshot data pipeline** (D-048 completion, P2.3-P2.5) AFTER the intake pipeline,
   targeting the convention build, not the August pilots.
6. **Phase 4 slimmed for the window** (D-056.3): minimal P4.1 story authoring under
   facilitator authority for the pilots; P4.2 primer and the bulk of P4.3 defer; P4.4
   snapshot acceptance rides item 5.
7. **P1.3 benchmark deferred to September** (post-pilot).

### Phase 0 - Workflow, congruence, and ratification foundation (must be first) [effort: M]

**Objective.** Make the verification battery a single enforced command, reconcile the
working tree, pin the calendar, and record the gates - so every downstream phase's exit
criteria mean something.

**Session kickoff.** Read CLAUDE.md, HANDOFF.md, and this plan's Current Position +
Decision Gates. Verify before touching anything: `git status` shows a single local
`main` with no remotes and MANIFEST.md/PLAN_1.0.md still untracked (`git ls-files
MANIFEST.md PLAN_1.0.md` returns empty); `ls scripts/` shows no `check-all` yet; the
existing `scripts/hooks/pre-commit` PII scan runs. Confirm Q-A (convention date) is still
unanswered before building any calendar.

- [ ] **P0.1** Pin the convention date: answer Q-A (date, registration window,
  consent-email lead time); if runway is insufficient, trigger the G-DATE fallback and
  record the re-scope.
  - *Touches:* HANDOFF.md (open gates), docs/PROJECT_PLAN.md section 3, DECISIONS.md.
  - *Verify:* date or fallback decision present in HANDOFF.md and DECISIONS.md.
  - *Depends:* none (this is microtask #1; G-DATE is owner-answered).
- [ ] **P0.2** Author `scripts/check-all` (single local orchestrator) running, in order:
  `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo
  test --workspace`, `wasm-pack build crates/cn-wasm --target web`, app
  `typecheck`/`build`/`test`, `validate:templates`, `smoke:node`, `build:snapshot`
  (content + size), and the PII scan.
  - *Touches:* new `scripts/check-all` (pwsh), AGENTS.md repo-commands, app/package.json
    scripts (`typecheck`, `build`, `test`, `validate:templates`, `smoke:node`,
    `build:snapshot` - all confirmed present), scripts/pii-scan.ps1, scripts/check-size.mjs.
  - *Verify:* `check-all` exits non-zero if any member fails; exits zero on a clean tree.
  - *Depends:* none.
- [ ] **P0.3** Enforce the battery as a gate without crossing the no-remotes gate: add a
  local pre-push (or expanded pre-commit) hook that runs `check-all`, keeping the existing
  staged PII scan on pre-commit.
  - *Touches:* scripts/hooks/ (pre-commit and/or new pre-push), scripts/install-hooks.ps1.
  - *Verify:* a deliberately failing test blocks the hook.
  - *Depends:* P0.2.
- [ ] **P0.4** Separate automated gates from human-only gates in AGENTS.md repo-commands:
  mark screenshot review, accessibility testing, convention rehearsal, and
  consent/language review as human-only, not automatable.
  - *Touches:* AGENTS.md (Repo commands section).
  - *Verify:* AGENTS.md distinguishes the two lists; `check-all` covers only the
    automatable set.
  - *Depends:* P0.2.
- [ ] **P0.5** Record the acceptance-unit inventory (this document's Current Position) as
  the single completion truth; stop using "half done."
  - *Touches:* HANDOFF.md (phase status).
  - *Verify:* HANDOFF.md phase status cites the inventory, not a fraction.
  - *Depends:* none.
- [ ] **P0.6** Record decisions on tree congruence: whether MANIFEST.md and PLAN_1.0.md
  become tracked authorities or stay review-only; defer the MANIFEST reorg proposal;
  affirm single-branch `main`.
  - *Touches:* DECISIONS.md, HANDOFF.md.
  - *Verify:* DECISIONS.md entries exist for each; MANIFEST reorg explicitly deferred.
  - *Depends:* none.
- [ ] **P0.7** Record G-RAT, G-DATE, and G-BACKUP as named gates in HANDOFF.md open gates
  with their defaults.
  - *Touches:* HANDOFF.md (open gates).
  - *Verify:* all three present with defaults.
  - *Depends:* P0.1 (carries the G-DATE outcome).

**Exit criteria.** `check-all` runs green as one command and is enforced by a local hook;
automated vs human-only gates are separated in AGENTS.md; the convention date is pinned or
the fallback is recorded; tracked/review-only status is decided; the gates are recorded.
Codex review round on the orchestrator script passed.

### Phase 1 - Constellation legible + Explore (Phase 3 remainder, part 1; gate-blind) [effort: L]

**Objective.** Turn the accepted base renderer into a readable, navigable graph and land
the explore surface - WITHOUT touching the core query contract (the routing semantics are
ratification-dependent and live in Phase 5).

**Session kickoff.** Confirm M0 is met: `scripts/check-all` exists and runs green, and the
local hook is active. Read docs/adr/ADR-004-renderer.md (the perf table and the 70ms p95),
docs/design/DESIGN_BRIEF.md, and HANDOFF.md phase status. Verify the base renderer still
runs the fixture through the Web Worker and that `app/src/ui/` is still README-only (this
phase creates it). Note reduced-motion is already honored in `app/src/main.ts` and must
stay honored.

- [ ] **P1.1** Labels: troika SDF labels, zoom-adaptive policy, density culling, cap
  tokens (grind HIGH).
  - *Touches:* new `app/src/viz/labels.ts`, app/src/viz/{nodes,config,index}.ts, theme
    tokens (schemas/theme-tokens.schema.json consumers).
  - *Verify:* both fixtures render legible labels; `check-all` green.
  - *Depends:* M0.
- [ ] **P1.2** Motion/focus polish: dual color-buffer focus dim/highlight, camera fly-to,
  idle drift, all with reduced-motion variants.
  - *Touches:* app/src/viz/{halos,camera,quality}.ts, app/src/state/reducer.ts
    (reduced-motion state).
  - *Verify:* reduced-motion path suppresses animation; screenshot review by the director
    (human-only).
  - *Depends:* M0.
- [ ] **P1.3** **[deferred to September, post-pilot; D-056.3]** Benchmark re-run with
  labels + halos + motion live against the p95 <= 50ms tail goal (S3-A2); record the
  measured number vs the recorded 70.3ms allocation.
  - *Touches:* app/spike (the standing regression benchmark, D-017), docs/adr/
    ADR-004-renderer.md (perf table).
  - *Verify:* measured tail recorded in the ADR-004 table; spike harness run.
  - *Depends:* P1.1, P1.2.
- [ ] **P1.4** Search box over existing `cn-graph` search (attribute hits only; no new
  query contract).
  - *Touches:* new `app/src/ui/` search component, cn-api `search` (via the WASM client).
  - *Verify:* query returns expected fixture hits; `check-all` green.
  - *Depends:* M0.
- [ ] **P1.5** Detail panel: TSDF code primary + plain-language secondary (D-032), one-line
  provenance for members (D-033), own-record indicators.
  - *Touches:* new `app/src/ui/` detail component, cn-api `entity_detail`.
  - *Verify:* detail renders from `entity_detail` for a fixture entity.
  - *Depends:* M0.
- [ ] **P1.6** Legend with the "adjusted for readability" indicator.
  - *Touches:* new `app/src/ui/` legend component, theme tokens.
  - *Verify:* legend reflects the fixture palette.
  - *Depends:* P1.1.
- [ ] **P1.7** Flat list/table reading projection over existing `search`/`entity_detail`
  (the down-payment on the deferred accessibility parallel-DOM, D-035; addresses the
  pure-3D-reveal failure mode).
  - *Touches:* new `app/src/ui/` flat-projection component, cn-api `search`/`entity_detail`.
  - *Verify:* flat projection lists the same entities the 3D scene shows for the same
    viewer scope.
  - *Depends:* M0. (Consumed later by P4.2 and Phase 8.)

**Exit criteria.** Both fixture groups render with legible labels; focus/motion respect
reduced-motion; benchmark tail recorded against the 50ms goal; search/detail/legend/flat
projection usable; **no change to the core query contract**; `check-all` green; Codex
review round passed; director screenshot review (human-only) recorded.

### Phase 2 - Persisted-format schemas + snapshot data envelope (foundation; gate-blind) [effort: M/L]

**Objective.** Close the schema drift and the empty-snapshot gap before anything persists
or ships. Nothing downstream (stories, ingest, export, snapshot acceptance) is built until
this phase's schemas exist and the snapshot proves it loads real data.

> **Resequencing note (2026-07-24, D-048/D-056.3).** P2.1/P2.2 landed. The snapshot
> DATA pipeline (P2.3-P2.5, decoupled and parked in D-048) is resequenced AFTER the
> intake pipeline and targets the CONVENTION build - the August internal pilots run
> the normal app build and do not wait on it. Remaining scope per D-048: main-thread
> `WasmTransport` so the artifact needs no external worker (D-046), `--public-layer`
> wired into `build:snapshot` (isolated non-tracked dir), per-artifact size gate in
> `check-size.mjs`, the no-leak acceptance test (D-047.4), then flip
> `CN_EMBED_SNAPSHOT` on by default.

**Session kickoff.** Confirm M0 is met. Read AGENTS.md I7/I8, schemas/README.md, the two
existing schemas (`schemas/group-template.schema.json`, `schemas/theme-tokens.schema.json`),
app/vite.config.ts (snapshot mode adds only `viteSingleFile()`), scripts/check-size.mjs,
and the `app/src/main.ts` DEV gate (line 63). Verify the current gap directly: a
`build:snapshot` today embeds no data, so demonstrate the empty-viewer failure before
closing it.

- [ ] **P2.1** Define versioned JSON Schemas for the persisted formats that are absent:
  the op-log/export envelope (required by AGENTS.md I7) and the story-path schema (named
  in schemas/README.md). Each carries an explicit `schema_version` (I7). (The intake
  contract schema lands with Phase 5 P5.3.)
  - *Touches:* new `schemas/*.schema.json` (op-log/export, story-path), schemas/README.md,
    app/scripts/validate-templates.mjs (`validate:templates`), cn-model/src/story.rs.
  - *Verify:* schemas validate the existing fixtures; added to `validate:templates`.
  - *Depends:* M0.
- [ ] **P2.2** Add unknown-major-version rejection as an explicit, tested behavior for
  every reader/importer (I7).
  - *Touches:* cn-store/cn-api reader paths, cn-schema, corresponding tests.
  - *Verify:* a fixture with a bumped major is rejected loudly by a test; no silent
    acceptance.
  - *Depends:* P2.1.
- [ ] **P2.3** Design the snapshot data envelope: viewer context, boot path, embedded
  `schema_version`, and a deterministic data-embedding step for snapshot builds (today
  `app/vite.config.ts` snapshot mode adds only `viteSingleFile()` and `main.ts` loads
  fixtures only under `import.meta.env.DEV`).
  - *Touches:* app/vite.config.ts (snapshot mode), app/src/main.ts (boot path), a new
    data-embedding step, the envelope schema from P2.1.
  - *Verify:* a snapshot build embeds fixture data; the boot path no longer depends on
    `import.meta.env.DEV`.
  - *Depends:* P2.1.
- [ ] **P2.4** Add a browser acceptance test (playwright headed, per D-017) that the
  snapshot loads meaningful fixture data offline and renders it - this must pass BEFORE the
  size gate counts as satisfied.
  - *Touches:* app/smoke or a new playwright test, scripts wiring into `check-all`.
  - *Verify:* playwright opens `dist/index.html` with no dev server and asserts a non-zero
    projected entity count.
  - *Depends:* P2.3.
- [ ] **P2.5** Choose and document the snapshot's baked-in viewer scope
  (Anonymous/Group-member, never facilitator or any Trusted/Private-bearing view;
  integration plan 6.4).
  - *Touches:* snapshot build config, a short docs note, cross-ref integration plan 6.4.
  - *Verify:* the build records the viewer scope; a test asserts no above-circle attribute
    is embedded.
  - *Depends:* P2.3.

**Exit criteria.** Versioned op-log/export and story schemas exist with unknown-major
rejection tests; the snapshot embeds and renders meaningful fixture data offline (browser
acceptance) with an explicitly documented viewer scope, then passes
`scripts/check-size.mjs`; `check-all` green; Codex review round passed.

### Phase 3 - Facilitator role, authority matrix, wizard, and entry forms (permission-adjacent; gate-blind) [effort: L]

**Objective.** Introduce facilitator authority in `cn-perm` BEFORE any authenticated story
mutation depends on it (the review's ordering fix; D-028 requires facilitator
entry/modification authority to live in `cn-perm`). Then let a facilitator stand up and
populate a group in-app without a developer.

**Session kickoff.** Confirm M0 is met. Read integration plan section 6.3
(`GroupRole::Facilitator` authority matrix), DECISIONS D-028/D-036, and the code the change
threads through: `core/crates/cn-model/src/group.rs` (GroupRole has only Member/Governance
today), `cn-perm/src/{viewer,authz,rules,projection}.rs`, and
`core/crates/cn-api/src/session.rs` (the `viewer_fingerprint` projection cache). Verify the
existing authz and no-leak property tests are green before touching the role model - this
is permission-adjacent work (grind HIGH + mandatory adversarial round).

- [ ] **P3.1** `GroupRole::Facilitator` in `cn-model/group.rs` + an
  `is_facilitator_or_governance` predicate in `cn-perm/viewer.rs` (alongside the existing
  `is_governance`), threaded through `authorize_op` (`cn-perm/authz.rs`).
  Permission-adjacent: director blueprint -> grind HIGH -> mandatory adversarial round.
  - *Touches:* core/crates/cn-model/src/group.rs, cn-perm/src/viewer.rs, cn-perm/src/authz.rs.
  - *Verify:* `cargo test --workspace` green including the authz suite.
  - *Depends:* M0.
- [ ] **P3.2** Explicit authority matrix (role x op-kind): facilitator CAN create/import
  unowned pilot records and author stories; CANNOT govern membership, grant roles, loosen
  visibility, lower a tier, or bypass owner controls. Note the custody-append tightening is
  NEW role/ownership logic (integration plan 6.3), not an inherited rule.
  - *Touches:* new authority-matrix doc under docs/, cn-perm/src/authz.rs tests.
  - *Verify:* matrix doc committed; each cell has a corresponding authz test.
  - *Depends:* P3.1.
- [ ] **P3.3** Fix the `active_role_names` match for the new variant
  (`cn-perm/src/viewer.rs`, currently exhaustive over Member/Governance) and confirm
  `viewer_fingerprint` distinguishes the role so the projection cache
  (`cn-api/src/session.rs`) never serves a facilitator a member's cached projection.
  - *Touches:* cn-perm/src/viewer.rs (`active_role_names`), core/crates/cn-api/src/session.rs.
  - *Verify:* a test proves distinct fingerprints for member vs facilitator on identical
    membership.
  - *Depends:* P3.1.
- [ ] **P3.4** Extend the no-leak property test to hold across
  anonymous/member/facilitator/self/governance.
  - *Touches:* cn-perm property tests.
  - *Verify:* property test green with the added role.
  - *Depends:* P3.1, P3.3.
- [ ] **P3.5** Facilitator group-setup wizard from JSON templates (D-036). **With P3.6,
  this IS the intake pipeline (D-053), not UI polish.**
  - *Touches:* new `app/src/ui/` wizard, fixtures/templates/research-network.template.json,
    cn-api submit/load.
  - *Verify:* wizard creates a group from `research-network.template.json` in-app.
  - *Depends:* P3.1.
- [ ] **P3.6** Template-driven entry forms (R1/R2/R3(a) UI): field widgets per attribute
  type, validation UX surfaced from `cn-schema` findings, draft handling - plus the
  **facilitator pending-review queue (D-053):** every submission (in-app or remote)
  lands pending and enters the graph only on facilitator approval; queue persistence is
  durable-write-first, gitignored staging, pii-scan covered, with near-duplicate
  surfacing (D-056.4); approved remote entries land unowned, facilitator-created
  (D-056.2). The remote leg (Pages form, sealed-box encryption, Workers/KV relay,
  pilot-PC puller) follows the ADR-005 draft (docs/adr/ADR-005-remote-intake.md) and
  its push/deploy preconditions.
  - *Touches:* new `app/src/ui/` entry forms + review queue, cn-schema validation
    reports (I12), gitignored staging store per ADR-005.
  - *Verify:* a fixture person is added through generated forms with validation
    surfaced; a pending submission is invisible to the graph until approved.
  - *Depends:* P3.5; the remote leg additionally on the ADR-005 round and the
    D-053/D-055 deploy preconditions.

**Exit criteria.** A facilitator creates a group from a template and adds people/entities
through generated forms with validation, all in-app; the authority matrix is documented and
exhaustively tested; the no-leak property holds across all five viewer classes; `check-all`
green; adversarial Codex round (permission-adjacent) passed.

### Phase 4 - Stories, comprehension layer, and snapshot acceptance (Phase 3 remainder, part 2; gate-blind) [effort: L]

**Objective.** Make the graph comprehensible to a lay audience and make the single-file
snapshot the primary acceptance vehicle. Depends on Phase 3 facilitator authority
(authoring is a facilitator-authorized mutation, not UI-only) and Phase 2 schemas +
envelope.

> **Resequencing note (2026-07-24, D-056.3).** Slimmed for the August pilot window:
> only a minimal P4.1 (story authoring under facilitator authority) ships for the
> pilots, with the D-044.4 mechanics/language split; P4.2 (primer) and the bulk of
> P4.3 defer past the window; P4.4 rides the resequenced snapshot pipeline and
> targets the convention build.

**Session kickoff.** Confirm M2 and M3 are met (Phase 2 story schema + snapshot envelope
exist and load real data; Phase 3 facilitator authority merged with authz tests green).
Read integration plan R6 and section 6.4, DECISIONS D-024/D-037, and
`core/crates/cn-model/src/story.rs` (the silent-elision behavior the load path relies on).
Verify the Phase 1 flat projection (P1.7) is in place, since the primer builds on it.

- [ ] **P4.1** In-app story authoring + viewing (D-037): facilitator (authorized per
  Phase 3) creates/edits/orders steps referencing entities by stable id; validated at load
  against the Phase 2 story schema; silent elision already in core.
  - *Touches:* new `app/src/ui/` story authoring/viewing, cn-model/src/story.rs, cn-api
    submit, the P2.1 story schema.
  - *Verify:* authoring is denied to a non-facilitator viewer by test; save/reload/play
    round-trips a story.
  - *Depends:* P3.1 (facilitator authority), P2.1 (story schema).
- [ ] **P4.2** Comprehension layer promoted to critical path (integration plan R6): an
  in-product "how to read this" primer.
  - *Touches:* new `app/src/ui/` primer, the P1.7 flat projection.
  - *Verify:* primer renders in the app.
  - *Depends:* P1.7.
- [ ] **P4.3** Seeded and tested Stories drawn from synthetic intake material + a written
  facilitator assembly-reveal script (docs deliverable).
  - *Touches:* fixtures/ (seeded stories), new reveal-script doc under docs/.
  - *Verify:* seeded stories load and play; script committed under docs/.
  - *Depends:* P4.1.
- [ ] **P4.4** Snapshot acceptance (D-024): embed a projection + theme for the Phase 2
  documented viewer scope; both fixture groups load and render from the snapshot; size
  budget green.
  - *Touches:* build:snapshot pipeline, both fixtures, the P2.3 envelope, the P2.4 browser
    test.
  - *Verify:* `build:snapshot` + the Phase 2 browser acceptance test both pass for both
    fixtures under 5MB.
  - *Depends:* P2.3, P2.4, P4.1.

**Exit criteria.** A facilitator can author, save, reload, and play a story under
facilitator authority (not UI behavior alone); primer + flat projection usable by a
non-graph-literate reader (director dry-run, human-only); snapshot builds, loads, and
renders both fixtures offline under 5MB with a documented viewer scope; `check-all` green;
Codex review round passed. **Product Phase 3 CLOSED** - R9 accessibility formally deferred
to v0.2 (D-035), with font-scale token, reduced-motion, and basic keyboard retained,
recorded as a DECISIONS deferral.

### Phase 5 - Routing semantics, ingestor, FORM, and pilot hardening -> v0.1.0 (scheduled; community-facing text human-gated) [effort: L/XL]

**Objective.** Land the routing contract and idempotent importer, finalize the
human-gated FORM, rehearse the full arc, and cut v0.1.0.

> **G-RAT resolved (2026-07-24, D-052):** the routing and importer semantics
> (P5.1-P5.3) are scheduled v0.1.0 work - docs/PROJECT_PLAN.md section 3 already
> commits them - against the D-052 real-usage acceptance bar (~150 expected, 300 max
> signups). The FORM follows D-051 (ATNI authors the vocabulary post-stability;
> standard developer language over the empty HSDS structure until then) and D-053 (the
> intake path is in-app entry + the sealed-envelope relay, not an external form
> platform); all community-facing text still parks on human review (D-023).

**Session kickoff.** Read HANDOFF.md open gates FIRST: community-facing text parks on
D-023 review, and any real ingestion additionally on the D-050 collective checkpoint. Read
integration plan sections 6.1 (`AtniIntakeBatchV0_1`) and 6.2 (routing contract), DECISIONS
D-030/D-034/D-023, and docs/design/pilot-form-and-template-2026-07-06.md. Verify Phase 2's
intake/story schemas exist and `core/crates/cn-ingest/src/lib.rs` is still a placeholder;
confirm the G-DATE answer and the pilot scope lock (or the near-date fallback) before
committing to the full arc.

- [ ] **P5.1** **[scheduled v0.1.0 work - G-RAT resolved, D-052]** Routing contract done
  properly, not "UI-only"
  (integration plan R3, spec 6.2): a concrete need-term -> candidate -> `query_paths`
  contract over a shared `tags` vocabulary; the returned path is the result (no opaque
  score, no LLM).
  - *Touches:* cn-graph, cn-api (`query_paths`/candidate resolution), integration plan 6.2.
  - *Verify:* a chosen need-term resolves to a rendered pathway on a fixture.
  - *Depends:* M3.
- [ ] **P5.2** **[scheduled v0.1.0 work - G-RAT resolved, D-052]** Structural
  contactability-consent gate in the
  cn-graph/cn-api candidate-resolution layer (not the UI, per I2): a non-consenting or
  facilitator-only person is never returned as a directly reachable endpoint.
  - *Touches:* cn-graph/cn-api candidate resolution, cn-perm (consent as a projection
    property).
  - *Verify:* a property test excludes non-consenting endpoints from search, paths, flat
    list, AND export.
  - *Depends:* P5.1.
- [ ] **P5.3** **[scheduled v0.1.0 work - G-RAT resolved, D-052]** `cn-ingest`
  `AtniIntakeBatchV0_1` importer (integration
  plan R2, spec 6.1): versioned contract (I7) against the Phase 2 intake schema, stable
  `source_row_id` idempotency key, deterministic UUIDv5 identity (not UUIDv7),
  first-sight-vs-resight semantics emitting `AttributeSet` on re-import, diffing against the
  prior imported source snapshot, `Origin::Ingested` + T1 tier (D-034), a first-sight
  `Imported` custody event (I12). Director blueprint -> grind -> adversarial round.
  - *Touches:* core/crates/cn-ingest/src/lib.rs, the Phase 2 intake schema (P2.1), cn-model
    provenance/tier constructors.
  - *Verify:* round-trip property test asserts zero new creates / zero duplicate edges /
    zero custody growth on identical re-import.
  - *Depends:* P2.1.
- [ ] **P5.4** `cn` CLI subcommands: ingest, validate, export, snapshot; the fast
  re-ingest -> snapshot-rebuild path for same-day QR joiners (D-030) as one facilitator
  command.
  - *Touches:* core/cli/src/main.rs, cn-ingest, the snapshot build.
  - *Verify:* CLI ingests a fixture, emits a validation report, rebuilds the snapshot in
    minutes on the reference laptop.
  - *Depends:* P5.3.
- [ ] **P5.5** **[vocabulary post-stability per D-051; text parks on human review
  D-023]** FORM deliverable finalized: intake form (now the in-app entry + D-053 relay
  form, not an external platform), ATNI template, consent text, QR/link mechanics,
  feedback plan. Per D-051, ATNI authors the capability vocabulary AFTER the system is
  stable; the structure ships with standard developer language until then. All
  community-facing text is human-gated (D-023) - now concretely needed for the relay
  form and the August pilots.
  - *Touches:* docs/design/pilot-form-and-template-2026-07-06.md, the ADR-005 relay
    form text.
  - *Verify:* structure retained with developer-language placeholders; human review
    recorded before any use.
  - *Depends:* human review (D-023); ATNI vocabulary sequenced post-stability (D-051).
- [ ] **P5.6** ADR-006 dedup (Session C; renumbered 2026-07-24 from ADR-005 - ADR-005
  is now the remote-intake ADR, D-056.1): always-queue review, exact-key handling,
  merge-vs-link, undo, idempotent re-ingest semantics; Fable ADR + one review round;
  implemented against fixtures with planted near-duplicates. Must reconcile with the
  D-056.4 intake-path dedup already in place by then: client-generated payload UUIDs
  plus facilitator near-duplicate surfacing in the pending-review queue.
  - *Touches:* new docs/adr/ADR-006-dedup.md, cn-ingest, planted-duplicate fixtures,
    the ADR-005 pending-queue dedup surface.
  - *Verify:* planted-duplicate fixture routes to the review queue, not a silent merge.
  - *Depends:* P5.3.
- [ ] **P5.7** CPF-RCN migration recipe (Session E, docs-only, D-031): export/scrub/tier/
  FPIC steps the human executes; the session never reads red data.
  - *Touches:* new docs/ recipe (docs-only).
  - *Verify:* recipe committed under docs/; no real data referenced.
  - *Depends:* none (docs-only; safe to run any time).
- [ ] **P5.8** Pilot hardening: perf pass at pilot-realistic scale with the ADR-004 table
  updated (re-measures the 70ms tail and the 2.9MB projection allocation in-browser);
  error/empty/loading-state UX sweep; facilitator guide (human reviews community-facing
  language).
  - *Touches:* docs/adr/ADR-004-renderer.md, docs/PROJECT_PLAN.md risk register (the 2.9MB
    line), app UX states, new facilitator guide under docs/.
  - *Verify:* measured numbers recorded; `check-all` green.
  - *Depends:* P5.1, P5.3.
- [ ] **P5.9** **Convention rehearsal (human-only):** full-arc dry run on the facilitator
  laptop with synthetic fixtures - ingest, reveal, route, story, re-ingest - against the
  primer, flat projection, and seeded Stories (integration plan R6 definition of done).
  - *Touches:* the full v0.1.0 surface; a dated rehearsal checklist artifact.
  - *Verify:* dated rehearsal checklist completed and recorded; this is the R6 acceptance,
    not a hoped-for outcome.
  - *Depends:* P5.3, P5.4, P4.4.
- [ ] **P5.10** Define the privacy-safe field-validation evidence artifact for the pilot (no
  participant data): a dated rehearsal checklist, a synthetic regression bundle,
  aggregate/non-identifying field observations, consented issue summaries, and a human
  sign-off record.
  - *Touches:* a new evidence-artifact template under docs/.
  - *Verify:* the artifact template exists and contains no PII; it becomes the
    field-validation gate feeding later 1.0 ratification (consumed by P9.6).
  - *Depends:* none (template definition; safe any time).

**Exit criteria.** `cn ingest` round-trips both fixtures losslessly with the idempotency
test green; routing resolves need-terms with the structural consent gate proven by test;
re-ingest + snapshot rebuild demonstrated in minutes; convention rehearsal completed;
snapshot under budget; human has reviewed all community-facing text; the privacy-safe
evidence artifact is defined; `check-all` green; full Codex review sweep since Phase 4
passed; **v0.1.0 tagged locally** against the D-052 real-usage bar (any push to the
public remote only after the D-053/D-055 preconditions: license in-repo, sweep passed,
core stability). The convention pilot then runs as the field validation feeding v0.2
scoping and the post-convention retrospective (D-052).

### Phase 6 - Personal mode (R4) [DEFERRED to the post-convention retrospective, D-052; effort: XL]

**Objective.** Deliver individual record ownership and per-attribute sharing - the
self-management the pilot deferred (D-029). Materially higher-discovery-cost than the front
half: permission-sensitive, new circle semantics.

**Session kickoff.** Confirm the post-convention retrospective (D-052) has ratified
Phase 6 into the 1.0 line (HANDOFF.md open gates) - if not, this phase stays a deferred
scenario and does not start. Read the Phase
5 exit re-scope (v0.2 scoping from the real pilot), DECISIONS D-029, CLAUDE.md R4, and
`cn-perm/src/{viewer,projection,rules}.rs` plus `cn-model/src/{circle,trust}.rs`. Verify the
five-viewer-class no-leak property test (from P3.4) is green before extending the circle
model.

- [ ] **P6.1** Profile ownership: an individual owns their record.
  - *Touches:* cn-model (ownership), cn-perm, cn-api submit.
  - *Verify:* ownership transfer test; `check-all` green.
  - *Depends:* post-convention retrospective ratification (D-052), M5.
- [ ] **P6.2** Per-attribute sharing UI across private/trusted/group/network/public,
  computed only in `cn-perm` (I2).
  - *Touches:* cn-perm/src/projection.rs, cn-model/src/circle.rs, new `app/src/ui/` sharing
    controls.
  - *Verify:* property test - no projection leaks an attribute above the viewer's circle for
    any of the five circles.
  - *Depends:* P6.1.
- [ ] **P6.3** Individual-managed trust grants + an audit log of grant changes.
  - *Touches:* cn-model/src/trust.rs, cn-store (op log), cn-api.
  - *Verify:* grant/revoke changes appear in the audit log.
  - *Depends:* P6.1.
- [ ] **P6.4** "View as" viewer-context switcher across every circle.
  - *Touches:* cn-api session/projection, new `app/src/ui/` viewer switcher.
  - *Verify:* switching viewer context re-projects correctly per circle.
  - *Depends:* P6.2.
- [ ] **P6.5** Firm up "network" circle semantics (recorded as provisional in the risk
  register).
  - *Touches:* cn-perm (circle boundary), a DECISIONS entry.
  - *Verify:* documented in `cn-perm`; tests cover the boundary.
  - *Depends:* P6.2.
- [ ] **P6.6** **Re-estimation gate:** after this phase, reforecast remaining 1.0 effort
  from the completed acceptance units.
  - *Touches:* HANDOFF.md, this plan (Milestones note).
  - *Verify:* reforecast recorded in HANDOFF.md.
  - *Depends:* P6.1, P6.2, P6.3, P6.4, P6.5.

**Exit criteria.** An individual owns a record, sets per-attribute visibility,
grants/revokes trust, and sees the graph as each viewer context; grant changes are audited;
the five-circle no-leak property test passes; `check-all` green; adversarial Codex round
passed.

### Phase 7 - Network-readiness and identity abstractions (R5) [DEFERRED to the post-convention retrospective, D-052; effort: L/XL]

**Objective.** Prove the network-readiness bet as a finalized, documented abstraction
without committing to a specific network (G-NET).

**Session kickoff.** Confirm the post-convention retrospective (D-052) has ratified
Phase 7 into the 1.0 line. Read ADR-002
(the event log, and A-B8 which reserves the `SyncTransport` contract for this phase),
DECISIONS D-027, and `core/crates/cn-sync/src/lib.rs` (still a placeholder). Verify the
op-log round-trip and idempotent-apply behavior in `cn-store` is green before building the
transport, and hold G-NET closed (design the abstraction, do not choose a network).

- [ ] **P7.1** Finalize the `cn-sync` `SyncTransport` trait with a local-only adapter (the
  contract ADR-002 A-B8 reserved for this phase).
  - *Touches:* core/crates/cn-sync/src/lib.rs, ADR-002 reference.
  - *Verify:* `cargo test` green for `cn-sync`.
  - *Depends:* post-convention retrospective ratification (D-052), M5.
- [ ] **P7.2** Validate op-log exchange between two local instances (UUIDv7 ids, HLC/LWW
  ordering, idempotent apply).
  - *Touches:* cn-sync, cn-store (op log fold).
  - *Verify:* two local instances reconcile a shared op log deterministically.
  - *Depends:* P7.1.
- [ ] **P7.3** Write the protocol-integration guide for a future network team.
  - *Touches:* new docs/ integration guide.
  - *Verify:* guide committed under docs/.
  - *Depends:* P7.2.
- [ ] **P7.4** Draft the identity ADR (D-027) framing options, with the
  protocol/identity/federation choice explicitly parked as G-NET.
  - *Touches:* new docs/adr/ identity ADR, HANDOFF.md (G-NET).
  - *Verify:* ADR drafted; choice parked.
  - *Depends:* P7.2.
- [ ] **P7.5** **Re-estimation gate** after the identity ADR.
  - *Touches:* HANDOFF.md.
  - *Verify:* reforecast recorded.
  - *Depends:* P7.4.
- [ ] **P7.6** **Open question flagged:** is two-local-instance reconciliation sufficient R5
  proof, or does honest validation require a real (gated) second party?
  - *Touches:* HANDOFF.md / Open Questions.
  - *Verify:* recorded for the human in Open Questions / HANDOFF.md.
  - *Depends:* P7.2.

**Exit criteria.** Two local instances reconcile a shared op log through `SyncTransport`;
the protocol-integration guide is written; the identity ADR is drafted with the network
choice parked as a human gate; `check-all` green; Codex review round passed.

### Phase 8 - Accessibility as primary interface + governance/tiering tooling (R9 full) [DEFERRED to the post-convention retrospective, D-052; effort: XL]

**Objective.** Raise accessibility from the pilot baseline (D-035) to the founding R9
standard, and build the governance/tier-enforcement tooling deferred from the pilot.

**Session kickoff.** Confirm the post-convention retrospective (D-052) has ratified
Phase 8 into the 1.0 line. Read DECISIONS
D-035 (what was retained vs deferred), D-028 (creator-governance handoff), D-034 (ATNI
Climate as tier authority), CLAUDE.md R9, and AGENTS.md I9. Verify the Phase 1 flat
projection (P1.7) is the foundation the parallel DOM builds on, and that the no-leak
property test still holds before adding the governance role.

- [ ] **P8.1** Parallel-DOM primary interface built on the Phase 1 flat projection.
  - *Touches:* new `app/src/ui/` parallel-DOM layer, the P1.7 flat projection.
  - *Verify:* the app is operable end-to-end via the parallel DOM.
  - *Depends:* post-convention retrospective ratification (D-052), P1.7, M5.
- [ ] **P8.2** Full keyboard navigation + ARIA labeling audit + `prefers-reduced-motion`
  coverage + 375px layouts (I9).
  - *Touches:* app/src/ui, app/src/viz motion paths.
  - *Verify:* keyboard-only and screen-reader passes (human-only accessibility testing)
    recorded.
  - *Depends:* P8.1.
- [ ] **P8.3** AA contrast/accessibility audit.
  - *Touches:* theme tokens, app/src/ui, an audit record.
  - *Verify:* audit passes or documents accepted exceptions.
  - *Depends:* P8.1.
- [ ] **P8.4** Governance role and membership tooling (creator-governance handoff, D-028).
  - *Touches:* cn-perm (governance authority), cn-model membership, new `app/src/ui/` admin.
  - *Verify:* governance role manages membership/roles with the no-leak property intact.
  - *Depends:* post-convention retrospective ratification (D-052), M5.
- [ ] **P8.5** Per-field tier UX and tier-enforcement development, ATNI Climate as tier
  authority (D-034).
  - *Touches:* cn-model/src/tier.rs, cn-perm exports, new `app/src/ui/` tier controls.
  - *Verify:* per-field tiering works; exports respect tiers.
  - *Depends:* P8.4.
- [ ] **P8.6** **Re-estimation gate** after accessibility testing.
  - *Touches:* HANDOFF.md.
  - *Verify:* reforecast recorded.
  - *Depends:* P8.2, P8.3, P8.5.

**Exit criteria.** The app is fully operable by keyboard and screen reader through the
parallel-DOM interface; the AA audit passes or documents accepted exceptions; the governance
role manages membership/roles with the no-leak property intact; per-field tiering UX works
and exports respect tiers; `check-all` green; human-only accessibility testing recorded;
Codex review round passed.

### Phase 9 - 1.0 hardening, reusability proof, and first sibling integration [DEFERRED to the post-convention retrospective, D-052; effort: XL + external coordination]

**Objective.** Consolidate R1-R10 into a release a second community can adopt on its own,
and demonstrate the integration payoff. External coordination (a real second adopter,
sibling-tool teams) makes this the highest-uncertainty phase.

**Session kickoff.** Confirm the post-convention retrospective (D-052) has ratified the
1.0 line and that Phases 6, 7, and 8 are all at exit. Read CLAUDE.md R1-R10 (the traceability anchors), integration plan section 8
(sibling-tool roadmap), and the Phase 5 privacy-safe evidence artifact (P5.10) now that a
real pilot has run. Verify both distribution modes build clean via `check-all` and re-raise
G-BACKUP before any tag work.

- [ ] **P9.1** Reusability proof: a non-ATNI community type set up end-to-end from a
  template by a non-engineer, exercising R1/R2/R3 without code changes.
  - *Touches:* fixtures/templates (a new community type), the wizard + importer surface.
  - *Verify:* a second community type is stood up and populated by a non-engineer
    (human-only acceptance recorded).
  - *Depends:* post-convention retrospective ratification (D-052), M6, M7, M8.
- [ ] **P9.2** Full R1-R10 traceability audit.
  - *Touches:* a new audit doc under docs/, CLAUDE.md requirement anchors.
  - *Verify:* every requirement maps to shipped, tested capability in an audit doc.
  - *Depends:* M6, M7, M8.
- [ ] **P9.3** Distribution polish for both the snapshot and normal app builds with the size
  budget enforced.
  - *Touches:* app build config, scripts/check-size.mjs, `build`/`build:snapshot`.
  - *Verify:* both modes build clean and under budget via `check-all`.
  - *Depends:* M8.
- [ ] **P9.4** Docs for group admins and individuals, human-reviewed for community-facing
  language.
  - *Touches:* new docs/ admin + individual guides.
  - *Verify:* guides committed and review recorded.
  - *Depends:* P9.1.
- [ ] **P9.5** First sibling-tool integration (cap-assessor) through the R2 importer and R1
  shared vocabulary.
  - *Touches:* cn-ingest (the shared contract), a cap-assessor import fixture, integration
    plan section 8.
  - *Verify:* cap-assessor rows import through the same contract into ATNI-authored
    vocabulary.
  - *Depends:* P5.3, P9.1.
- [ ] **P9.6** Field-validation gate: the Phase 5 privacy-safe evidence artifact is
  populated from the real pilot and signed off.
  - *Touches:* the P5.10 evidence artifact.
  - *Verify:* evidence artifact complete, no PII.
  - *Depends:* P5.10.
- [ ] **P9.7** Re-raise G-BACKUP as a release blocker to reconsider before tagging.
  - *Touches:* HANDOFF.md, DECISIONS.md.
  - *Verify:* human decision recorded.
  - *Depends:* none (owner-answered, before tag).

**Exit criteria.** A second community type is stood up and populated by a non-engineer;
every R1-R10 requirement maps to shipped, tested capability; both distribution modes build
clean and under budget; admin and individual guides exist and are human-reviewed; at least
one sibling tool imports through the shared contract; the field-validation evidence is
signed off; the final Codex review sweep is clean; **1.0 tagged** (remote/publishing
decisions remain human gates).

## Milestones

Each milestone's exit evidence is an unambiguous artifact: a command output, a committed
file, or a recorded decision. The Depends column names the milestones and gates a milestone
requires before it can close.

| Milestone | Phase | Exit evidence (command output / committed artifact / recorded decision) | Depends | Effort class |
|---|---|---|---|---|
| M0 Verification + congruence foundation | 0 | `scripts/check-all` exits 0 on a clean tree and non-zero on a planted failure; the local hook blocks that failure; AGENTS.md lists automated vs human-only gates; DECISIONS.md carries the tracked/review-only + reorg-deferral + single-`main` entries; HANDOFF.md pins the date (or records the G-DATE fallback) and lists G-RAT/G-DATE/G-BACKUP with defaults | - | M |
| M1 Legible navigable constellation | 1 | Both fixtures render legible labels (screenshot committed); reduced-motion suppresses animation (test); ADR-004 table shows the re-measured tail vs the 50ms goal; search/detail/legend/flat projection usable; `check-all` green with no core query-contract diff | M0 | L |
| M2 Persisted schemas + live snapshot envelope | 2 | New op-log/export + story schemas committed with `schema_version`; unknown-major rejection test red-on-bump; playwright asserts a non-zero projected entity count from `dist/index.html` offline; documented viewer scope + no-above-circle-attribute test; size gate green | M0 | M/L |
| M3 Facilitator authority | 3 | `GroupRole::Facilitator` + `is_facilitator_or_governance` merged; authority-matrix doc committed with a test per cell; distinct member-vs-facilitator `viewer_fingerprint` test; no-leak property green across five viewer classes; wizard builds a group from `research-network.template.json` in-app | M0 | L |
| M4 Comprehensible reveal + accepted snapshot | 4 | Story authoring denied to non-facilitator (test) and save/reload/play round-trips; primer renders; seeded stories load and play + reveal script committed; `build:snapshot` + browser acceptance pass for both fixtures under 5MB; DECISIONS deferral for R9 recorded | M2, M3 | L |
| M5 v0.1.0 convention-pilot ready | 5 | Idempotency property test green (zero-new-create re-import); routing + structural consent gate proven by test; `cn` CLI ingest/validate/export/snapshot rebuilds in minutes; FORM human-review recorded; dated rehearsal checklist committed; `git tag v0.1.0` present locally | M4; human review (D-023) + the D-050 collective checkpoint for any real ingestion (former G-RAT/G-DATE/G1 deps RESOLVED, D-050/D-051/D-052) | L/XL |
| M6 Personal mode | 6 | Ownership-transfer test; five-circle no-leak property green; grant/revoke audit-log entries present; "view as" re-projects per circle (test); reforecast recorded in HANDOFF.md | M5; post-convention retrospective (D-052) | XL |
| M7 Network + identity abstractions | 7 | Two local instances reconcile a shared op log via `SyncTransport` (deterministic test); protocol-integration guide committed; identity ADR drafted with G-NET parked; reforecast recorded | M5; post-convention retrospective (D-052), G-NET (parked) | L/XL |
| M8 Accessibility + governance tooling | 8 | Keyboard-only + screen-reader passes recorded (human-only); AA audit committed (pass or accepted exceptions); governance manages membership with no-leak intact (test); exports respect per-field tiers (test); reforecast recorded | M5, M1; post-convention retrospective (D-052) | XL |
| M9 1.0 release | 9 | Non-engineer stands up + populates a second community type (recorded); R1-R10 traceability audit committed; both distribution modes under budget via `check-all`; admin + individual guides committed with review recorded; cap-assessor rows import through the shared contract (test); field-validation evidence signed off, no PII; `git tag 1.0` (human release/publish gates separate) | M6, M7, M8; post-convention retrospective (D-052), G-BACKUP (re-raised) | XL + external coordination |

> Effort classes are deliberate ranges, not false precision. The project's own 9-12
> focused-session estimate covers only Phases 0-5 (v0.1.0) and is a **capacity hypothesis**,
> not a commitment: it rests on two sessions plus one highly automated day (67 commits,
> single local `main`, latest 2026-07-06). Track completed acceptance units and actual
> focused hours over the next several sessions and reforecast, keeping machine-parallel
> implementation time separate from human decision/review/accessibility/community-coordination
> time. Phases 6-9 carry re-estimation gates at their exits (P6.6, P7.5, P8.6).

## Provenance

This plan was produced by a four-phase pipeline, each phase handing a more-articulated
artifact to the next. The pipeline alternates a fast broad model (Codex) with a
deep-judgment model (Opus) so that inventory and adversarial pressure come from one and
synthesis and expansion from the other.

1. **Codex manifest.** Codex inventoried the working tree and produced MANIFEST.md - the
   file/directory inventory, the "Proposed Reorganization (NOT EXECUTED)" mapping, and the
   dated status snapshot that grounds this plan's Current Position.
2. **Opus draft (v0.1).** Opus drafted the first Plan-to-1.0 from the manifest plus the
   project's own docs (CLAUDE.md, docs/PROJECT_PLAN.md section 3, HANDOFF.md, the
   integration plan, DECISIONS.md, the ADRs).
3. **Codex adversarial review + refinement (v0.2).** Codex adversarially reviewed the
   draft against the repository's accepted planning authority; verdict **conditional no-go**
   - a useful draft roadmap, not yet congruent with the repo's own accepted authority. The
   fifteen challenges below were resolved into v0.2 (decision gates, the gate-to-microtask
   blocking map, effort classes, phase-ordering fixes, and the acceptance-unit inventory).
4. **Opus expansion (v1.0 - final; this document, 2026-07-11).** Opus expanded v0.2 into
   its final form: concrete anchors (touches/verify/depends) on every microtask, per-phase
   session-kickoff notes, the Risks and Mitigations and Plan Congruence sections, sharpened
   milestone exit evidence with a dependency column, and this Provenance section. Every
   contested claim, path, module name, and count was re-verified against the working tree
   on 2026-07-11 and corrected where it had drifted (for example: attributing the op-log/
   export schema requirement to AGENTS.md I7 rather than schemas/README.md; pinning the
   ADR-004 tail to the recorded 70.3ms).
5. **Phase 5 reconciliation (v1.1, 2026-07-24).** After the human's gate-grill session
   and the two-agent look-back (DECISIONS D-050..D-056), this pass reconciled the plan
   with the rulings: G-RAT/G-DATE/G1/Q-B/Q-C/P0.6 marked RESOLVED, the blocking map
   rewritten, the public-remote and tracked-docs framing corrected, the D-056
   resequencing recorded (normal-build August pilots, intake pipeline as P3.5/P3.6 +
   the ADR-005 relay, snapshot after intake targeting the convention build, Phase 4
   slimmed to minimal P4.1, P1.3 deferred to September), and the dedup ADR renumbered
   to ADR-006. Gate, sequencing, and status framing only; the 2026-07-11 code
   inventory was not re-verified.

### Challenge and resolution log (Phase 3 adversarial review)

Each challenge Codex raised and what changed to resolve it. Preserved verbatim from the
v0.2 review record; these are the changes that survive into this final plan.

1. **1.0 definition and plan-v3 ratification (blocker).** The draft made the inferred 1.0
   boundary operative. Changed: added DECISION GATE G-RAT; v0.1.0 is the committed
   deliverable; Phases 6-9 (and Phase 5's routing/ingest specs) are provisional pending
   ratification.
2. **Convention date and calendar (blocker).** The v0.1.0 arc had no calendar anchor (Q-A
   unpinned). Changed: G-DATE with the date pin as Phase 0 microtask #1 (P0.1) and a stated
   near-date fallback to a minimum rehearsalable pilot.
3. **Human gates and blocking map (blocker).** Phase 4 named the gates without a blocking
   matrix. Changed: added the Gate-to-microtask blocking map (gate-blind vs parked) and
   tagged parked microtasks in Phase 5.
4. **Backup risk / H: mirror (major).** Changed: G-BACKUP offers an optional maintainer-run
   local H: mirror as interim mitigation that does not cross the no-remotes gate, with the
   decision and the device-verification evidence reserved to the human; no autonomous
   action.
5. **Back-half effort realism (major).** Phases 5-8 had no effort estimate. Changed: effort
   classes on every phase (Phases 6-9: XL / L-XL / XL / XL+coordination) plus re-estimation
   gates after the pilot, the identity ADR, and accessibility testing.
6. **Invariant duplication and punctuation (minor).** Changed: replaced generic invariant
   restatements with the single line "All work is governed by AGENTS.md I1-I12," keeping
   only requirement-specific criteria (I7 unknown-major, I2 structural consent gate, I8 size
   gate); hyphens only.
7. **Phase ordering - facilitator authority after story authoring (major).** Changed:
   facilitator role + authority matrix in `cn-perm` moved to Phase 3, ahead of the
   authenticated story authoring now in Phase 4; the authoring exit criterion closes on
   facilitator authority, not UI behavior alone.
8. **Phase ordering - provisional routing contract in Phase 1 (blocker).** Changed: Phase 1
   is now ratification-independent renderer/search/flat-projection work with an explicit "no
   core query-contract change" exit criterion; the routing semantics moved to Phase 5 and
   park on G-RAT.
9. **Claimed completion state vs actual (major).** Changed: replaced "half done" with an
   acceptance-unit inventory (foundations + base renderer complete; workflows, snapshot
   payload, facilitator authority, ingest, CLI, rehearsal absent), recorded as the single
   completion truth in Phase 0.
10. **Snapshot data-embedding dependency (major).** Verified: `main.ts` DEV-gates the only
    fixture load (line 63); snapshot mode embeds no data; the size check proves bytes only.
    Changed: Phase 2 adds a snapshot data-envelope design task and a browser acceptance test
    that must pass before the size gate counts.
11. **Schema and persisted-format scope gap (major).** Verified: only group-template +
    theme-tokens schemas exist; schemas/README.md names more and AGENTS.md I7 requires
    op-log/export/story schemas. Changed: Phase 2 defines versioned op-log/export/story
    schemas with unknown-major rejection before stories, ingest, export, or the snapshot
    envelope are built.
12. **Single-maintainer throughput evidence thin (major).** Changed: the 9-12-session figure
    is labeled a capacity hypothesis with an instruction to track acceptance units and
    focused hours and reforecast, separating machine time from human time.
13. **1.0 field-validation evidence boundary undefined (major).** Changed: Phase 5 defines a
    privacy-safe evidence artifact (dated rehearsal checklist, synthetic regression bundle,
    aggregate observations, consented issue summaries, human sign-off, no PII) that becomes
    the Phase 9 field-validation gate.
14. **CI/test-gate enforcement gap (major).** Verified: only the staged PII scan is enforced
    (`core.hooksPath = scripts/hooks`); no CI. Changed: Phase 0 builds the single
    `check-all` orchestrator, enforces it via a local hook (no remote needed), and separates
    automated from human-only gates; no later exit criteria are reliable until this lands.
15. **Working tree vs planning-doc congruence (major).** Changed: added the Working-tree
    congruence section and Phase 0 microtasks to decide tracked vs review-only status, update
    the accepted docs atomically on ratification, keep single-branch `main`, and defer the
    MANIFEST reorg until after the pilot critical path.

## Open Questions

Only items still genuinely open (the rest are now encoded as DECISION GATES or plan
structure); entries answered in the 2026-07-24 gate-grill are retained below with
RESOLVED markers for traceability:

1. **R5 network-readiness proof sufficiency.** Is two-local-instance reconciliation (Phase
   7) adequate proof of R5, or does honest validation require a real second party (itself a
   G-NET-gated commitment)? (Flagged as P7.6.)
2. **Unvalidated performance and size claims.** The p95 70.3ms tail vs the 50ms goal
   (ADR-004) and the 2.9MB projection JSON per recompute at 5k scale
   (docs/PROJECT_PLAN.md risk register) are allocations, not in-browser measurements. P1.3
   (deferred to September, D-056.3) and P5.8 re-measure; if the tail or projection size
   does not hold at pilot-plus scale, ADR-003 D2's typed-array escape hatch or renderer
   rework re-enters scope.
3. **Sibling-tool integration breadth in 1.0.** This plan places only cap-assessor inside
   1.0 (P9.5) and defers TCR-policy-scanner, GeoBase, and engagement-database. A reviewer may
   argue the shared taxonomy/TSDF/graph spine is either more central (pull integrations
   earlier) or entirely out of 1.0 scope. Part of the post-convention retrospective's
   deferred ratification (D-052).
4. **Product name vs folder name (D-001).** "Community Navigator" (product) vs
   `community-connector` (folder) is unresolved; a 1.0 release should settle the public name.
5. **Form platform (Q-B).** RESOLVED 2026-07-24 (D-053): no external form platform.
   Direct in-app entry plus the QR sealed-envelope relay (with the facilitator
   pending-review queue) is the intake path; CSV ingestion is a secondary path for
   structured sources, keeping the generic column-mapping importer stance.
6. **License variant (Q-C).** RESOLVED 2026-07-24 (D-054): PolyForm Noncommercial
   1.0.0, tracked as LICENSE.md before the first push to the public remote.
