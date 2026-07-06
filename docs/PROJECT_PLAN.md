# Community Navigator - Project Plan to v0.1.0

> Durable plan. Read after CLAUDE.md. Live state stays in HANDOFF.md; this
> document changes only when the plan itself changes (log a DECISIONS entry
> when it does). Companion: docs/NEXT_SESSION.md (resume brief + the human
> interview). Last revised: 2026-07-06, session 2 (decision session;
> Execution Plan v2 per D-039).

## 1. Where the project stands

Phases 0-2 are COMPLETE: contract docs and PII tripwire; ADR-001..003
accepted through adversarial rounds; the full Rust/WASM core (six crates,
permission property test, measured 255ms fold / 133ms projection at
5k+10k); closing review with all accepted findings fixed.

Phase 3 is roughly HALF done: ADR-004 renderer decision measured on the
reference GPU; design brief final; state machine (I4); theming pipeline
with CVD-clean fixture palettes; base renderer landed, visually fixed and
director-accepted (docs/design/screenshots/viz-fixes-2026-07-06.png).

The 2026-07-06 decision session (D-019..D-040) defined the pilot: an
ATNI Climate convention arc, facilitator-run, snapshot-first, with
need-to-solution routing as a hero workflow. Personal mode, identity,
deep accessibility, and governance tooling moved to v0.2. Section 3
below is Execution Plan v2.

## 2. The operating model (what makes this project fast)

Two engines with strictly separated roles, proven over ~20 offloaded tasks:

**Fable (director)** - the judgment layer. Writes blueprints and ADRs,
judges every Codex output against spec, runs spot-check greps that have
repeatedly caught real defects (I2/I4 violations, shallow revisions, its
own blueprint errata), frames human gates, talks to the human, commits.
Fable never writes bulk implementation and never re-verifies what a grep
can check.

**Codex (offload)** - the throughput layer, two profiles:
- grind (workspace-write): implementation from blueprints, test authoring,
  iterate-to-green loops, bulk transforms.
- review (read-only): adversarial rounds. EVERY round run so far returned
  accepted blocking findings - this is the highest-ROI pattern in the
  project; never skip it on ADRs or phase closes.

**Routing rules (validated; supersede intuition):**
| Work | Route | Notes |
|---|---|---|
| ADRs, blueprints, gate framing, human communication | Fable | The leverage point; short dense documents |
| Implementation from a blueprint | codex grind gpt-5.5 medium | HIGH for shaders, concurrency, permission-adjacent code |
| Single-file mechanical transforms only | gpt-5.4-mini low | D-014: anything multi-ruling needs 5.5 |
| Adversarial review, audits, research memos | codex review gpt-5.5 high | Output caps (800-1800 words); findings judged, never auto-applied |
| Verification | Codex runs checks -> Fable re-runs checks + targeted greps | Both, always; hooks enforce PII structurally |
| Visual/interaction acceptance | Fable + playwright headed Chrome | Screenshots into docs/design/screenshots/ |

**Session mechanics:** blueprints precise enough that grind needs no
judgment; ambiguity notes flow back for rulings (twice the implementer
correctly caught director errata - keep that loop). Detached codex
processes survive session ends; one-shot crons are the wake mechanism
(session-only - they die with the terminal, so HANDOFF must always carry
the chain). Never point --output-last-message at a task's own artifact.

**Session shapes:**
1. BUILD (default): read HANDOFF -> author/refresh 1-3 blueprints ->
   launch codex chain -> judge/commit as results land -> update HANDOFF +
   NEXT_SESSION brief. Fable tokens go to blueprints and judgment only.
2. REVIEW: codex review sweep -> Fable rulings -> directed fix task ->
   verify -> phase close (the Phase 2 pattern).
3. DECISION: run the NEXT_SESSION interview with the human -> convert
   answers to DECISIONS/ADR entries -> unblock build sessions.
4. ACCEPTANCE: drive the real app (playwright), measure against criteria
   (R9 audit, perf benchmark, snapshot budget).

## 3. Execution Plan v2 (decision session 2026-07-06; D-039)

> v0.1.0 = "convention-pilot ready": a facilitator-run, snapshot-first
> build that can ingest intake-form CSVs, render the participant +
> committee graph, route needs to solutions, and present authored
> stories at the ATNI Annual Convention. Personal mode, identity, deep
> accessibility, and governance tooling move to v0.2 (post-pilot).
> PROVISIONAL: a plan-v3 sitting follows the graph-networks research
> report (D-040); expect refinement, not reversal.

### The pilot arc (fixed points this plan serves; D-022, D-030)

1. Consent email -> intake form (QR codes in convention packets)
2. Ingest form-export CSV (T1 default, provenance stamped, dedup queue)
3. General assembly: full graph reveal (snapshot, facilitator laptop)
4. Committee meeting: need-to-solution pathway exploration; same-day QR
   joiners appear via fast re-ingest + snapshot rebuild
5. Feedback capture -> v0.2 scoping

### Phase 3 remainder - Frontend

**S3-A2 "Constellation legible"** (next build session)
- Labels: troika SDF, zoom-adaptive policy, density culling, cap tokens.
  (grind HIGH - fiddly.)
- Motion: focus-mode dim/highlight via the dual color buffers, camera
  fly-to polish, idle drift, all with reduced-motion variants. (grind
  medium from brief section 3.)
- Acceptance: screenshot review by Fable; spike benchmark re-run with
  labels+halos live (record vs the p95 <= 50ms tail goal).

**S3-B "Explore + Route"**
- View modes from template data; search UI over cn-graph search; legend
  with the "adjusted for readability" indicator.
- Detail panel: entity_detail path, own-record indicators, TSDF tier
  codes primary with plain-language secondary (D-032), one-line
  provenance for members with full custody chain for governance (D-033).
- NEED-TO-SOLUTION ROUTING UI (hero workflow, D-021): "who here can help
  with X" over cn-graph constrained paths; pathway rendering in the 3D
  scene + a readable path summary. Fable designs the routing UX first;
  grind medium-high implements.

**S3-C "Stories"**
- In-app story AUTHORING + viewing (D-037): facilitator composes stories
  (create/edit/order steps referencing entities); validated at load;
  silent elision already in core.
- Snapshot acceptance: embed a projection + theme; both fixture groups
  load and render; size budget green (D-024 makes this primary-vehicle
  acceptance, not a side check).

**Session B "Groups without engineers"**
- Facilitator group-setup wizard from JSON templates (D-036) +
  template-driven entry forms (R1/R2/R3 UI): field widgets per attribute
  type, validation UX from cn-schema findings, drafts.
- Facilitator/developer role added to cn-perm (D-028): entry/modification
  authority now, designed to hand off to group-creator governance later.
  Permission-adjacent = grind HIGH + adversarial review.

**DESIGN sitting (human-present; scheduled by the human; D-038)**
- Focused design agents + claude design pass on Hearthlight; cultural
  palette/shape review; the parked brief critique money goes here.
  Before convention polish.

**Phase 3 close**
- Codex review sweep + rulings + fixes. R9 accessibility criteria
  DEFERRED per D-035 (font-scale token, reduced-motion, basic keyboard
  retained; parallel-DOM primary interface + AA audit move to v0.2
  Session A). Phase 3 CLOSED.

### Phase 4 - Ingestor + pilot ops

**FORM (docs, human-reviewed) "Intake form + ATNI template"**
- Co-design the intake form and the ATNI climate-resilience group
  template - one artifact, two views (D-023): shared offer/need taxonomy,
  edge-generating questions, capacity + contactability, per-field
  visibility consent, core + depth structure.
- Consent email draft; QR/link mechanics; feedback-capture plan.
- Human reviews ALL community-facing text. Runs EARLY - it gates the
  consent email and needs no code.

**Session C (light) "Dedup" -> ADR-005**
- Always-queue review (Q6.2 default), exact-key handling, merge-vs-link
  ops, undo; idempotent re-ingest semantics (D-030). Fable ADR + one
  review round; grind implements against fixtures with planted
  near-duplicates.

**S4-A "Ingestor + CLI"**
- cn-ingest importers (CSV first: intake-form export + generic roster;
  then JSON/YAML) -> ops with provenance + T1 default tier (D-034);
  review queue; fixture round-trip lossless.
- cn CLI subcommands: ingest, validate, export, snapshot.
- FAST RE-INGEST -> snapshot rebuild path (same-day QR joiners, D-030):
  idempotent re-run, minutes not hours, one command for the facilitator.

**Session E (docs-only) "CPF-RCN migration recipe" (D-031)**
- Export/scrub/tier/FPIC steps the human executes; the session never
  reads red data. Execution human-gated.

**S4-B** - Phase 4 closing review + fixes. Phase 4 CLOSED.

### Phase 6 - Hardening -> v0.1.0 (Phase 5 vacates v0.1.0 per D-029)

- Perf pass at pilot-realistic scale; benchmark re-run; ADR-004 table
  updated.
- Error-state UX sweep; empty states; loading states.
- Facilitator guide (codex drafts, Fable voice pass, HUMAN reviews
  community-facing language). Individual guide moves to v0.2 with
  personal mode.
- CONVENTION REHEARSAL: full pilot arc dry run on the facilitator laptop
  with fixtures - ingest, reveal, route, story, re-ingest.
- Full codex review sweep over everything since Phase 2 close -> rulings
  -> fixes -> tag v0.1.0 locally (remotes remain gated per D-026).

Estimated remaining to v0.1.0: 9-12 focused sessions.

### v0.2 (post-pilot; shaped by feedback + the research sitting)

Personal mode (Phase 5: S5-A profile ownership/sharing, S5-B cn-sync +
protocol guide), Session D identity ADR (D-027), Session A accessibility
as primary interface (D-035), Session F sovereignty/governance tooling +
tier enforcement development (D-034), story authoring polish, and
integration spikes with the sibling tools (cap-assessor,
TCR-policy-scanner, GeoBase, engagement-database; D-040).

## 4. Risk register (standing)

| Risk | Mitigation |
|---|---|
| Single-machine repo: disk failure loses everything | Risk ACCEPTED by the human (D-026); remotes gate closed. Re-raise at every decision session |
| Convention date not pinned in-repo | Confirm date + registration-window reality with the human (NEXT_SESSION) |
| Form response rate = graph quality | Short required core, facilitator-assisted completion, QR joins at the convention (D-023, D-030) |
| Same-day re-ingest turnaround at the convention | Explicit S4-A requirement + Phase 6 rehearsal |
| Codex sandbox bypass (D-008) | Blueprint-constrained tasks + director verification; revisit if Codex behavior ever surprises |
| Projection JSON 2.9MB per recompute at 5k scale | ADR-003 D2 typed-array escape hatch; re-measure in-browser before Phase 6 |
| p95 tail (70ms) vs 50ms goal unvalidated | S3-A benchmark re-run; halo mitigation is the named lever |
| "Network" circle semantics provisional | Firms up with the (human-gated) network ADR; documented in cn-perm |
| Report-type duplication (cn-schema/cn-store) | Recorded debt; unify when cn-ingest lands (S4-A) |
| Session-only crons die with the terminal | HANDOFF always carries the chain; NEXT_SESSION.md carries the human-facing resume |
