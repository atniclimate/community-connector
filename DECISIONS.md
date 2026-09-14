# DECISIONS.md - Judgment Calls, Ladder Climbs, Adversarial Outcomes

Newest entries at the bottom. Every deep-thinking ladder climb past rung 1, every
adversarial Codex round outcome, and every nontrivial autonomous decision lands here.

---

## D-001 (2026-07-06) - Product name vs folder name

The brief names the product **Community Navigator** but mandates the folder
`C:\dev\community-connector`. Proceeding with product name Community Navigator in
docs and package names, folder as mandated. **Human may rename either later;**
flagged in HANDOFF.md. No code depends on the folder name.

## D-002 (2026-07-06) - Default branch `main`

`git init -b main`. The predecessor used `master` with a configured default of
`main`, which caused tooling friction. Reversible; standard.

## D-003 (2026-07-06) - Git identity set deliberately, pending confirmation

Predecessor commits auto-resolved to a machine-resolved tenant identity nobody
chose (address redacted for publication, D-059 - exactly the failure the brief
flags). This repo sets
in-repo config: `Patrick Freeland <accounts@indigenousaccess.org>` (the account
identity of this machine's Claude session). **Human gate-adjacent: confirm or
correct in HANDOFF.md.**

## D-004 (2026-07-06) - docs/CODEX_GUIDE.md reconstructed, not copied

The launch prompt says to copy the Codex guide into the repo, but no source
CODEX_GUIDE.md exists anywhere under C:\dev or the user's .claude directory
(verified by glob). The guide was authored fresh from the launch prompt's
references to its contents (operating model, SPECIALIZE blocks, escalation ladder
as its section 4, cost discipline, recipes) plus current `codex --help` output.
If the human has the original guide, drop it in and reconcile; differences should
be treated as contract-doc contradiction (stop-the-line rule).

## D-005 (2026-07-06) - Usage-limit failover rule (human directive)

Mid-session human directive: when Claude usage limit reaches ~98%, send the job to
Codex until the limit resets, then resume. Encoded in CLAUDE.md "Usage failover".
Mechanically: mechanical/implementation work goes to the `grind` profile,
verification to `review`; director-level judgment work parks in HANDOFF.md rather
than being delegated, because Codex does not hold the contract context.

## D-006 (2026-07-06) - Design research run as a parallel multi-agent workflow

Per human directive, a 5-researcher + synthesis + adversarial-critique workflow
(run id wf_afb4e38d-a45) produced `docs/design/DESIGN_BRIEF.md` to drive Phase 3
visual/motion direction. The brief is advisory input to Phase 3, not a contract
doc; conflicts with invariants resolve in favor of AGENTS.md.

## D-007 (2026-07-06) - Early usage failover: bootstrap tail delegated to Codex

The Claude session usage limit was hit mid-bootstrap (the design workflow's
critique agent died on it; reset 3:10am America/Los_Angeles). Per the human's
directives: (a) cargo workspace + app shell scaffold delegated to detached
codex grind run (.codex/task-scaffold.md); (b) workflow token-conservation
analysis delegated to codex review (.codex/task-token-analysis.md); (c) session
resume scheduled via in-session cron at 3:12am local - two minutes after reset
so the new limit window is definitely active. Director-judgment work (ADR-001,
fixtures content, brief revision) parked in HANDOFF.md, not delegated.
Postscript: the reset turned out to be 3:10am the SAME night (minutes away);
the stale next-day cron was deleted and the director resumed directly.

## D-008 (2026-07-06) - Codex Windows sandbox broken; bootstrap runs use bypass

Ladder rung 2 (root cause). Trigger: first codex exec hung 10+ minutes; after
adding project trust, every sandboxed shell call failed with
`windows sandbox: runner error: CreateProcessAsUserW failed: 5` (access denied;
codex config has `[windows] sandbox = "elevated"`, and this context cannot
spawn the elevated runner). Options: (a) run codex from an elevated shell -
untested, needs the human; (b) fix windows sandbox config - no documented
non-elevated mode found in the config reference; (c) `--sandbox
danger-full-access` per run. Chose (c) for bootstrap: trusted local repo,
blueprint-constrained tasks, approval never. Strongest surviving objection: a
misbehaving Codex run has full user-account access; mitigated by strict task
files, no-git rules, and director re-verification of all outputs. Revisit with
the human whether to fix the elevated sandbox properly.

## D-009 (2026-07-06) - Standing Claude/Codex routing policy adopted

Codex review session 019f36f7-6e6e-7032-9c13-3bb8f8cfeb7e analyzed the
design-research workflow (460k subagent tokens): biggest avoidable sinks were
five overlapping Claude research lanes and full-JSON re-serialization of all
research into the synthesis prompt (~151k chars). Policy table now in
docs/CODEX_GUIDE.md section 7; full analysis preserved at
docs/analysis/token-analysis-2026-07-06.md. Estimated savings had it been in
force: 380-450k Claude tokens. Also learned mechanically: never point
--output-last-message at a task's own artifact path (clobbers it; recovered
this one from the run log).

## D-010 (2026-07-06) - ADR-001 adversarial round 1: accepted with amendments

Codex review session 019f36fa-a523-7ee3-8e3a-fdb3d9afced4 returned 6 blocking
objections, 6 advisories, verdict ACCEPT-WITH-AMENDMENTS. All six blockers were
real (query-closure leaks, undefined tier x circle cells, missing
attribute-level tier, story leakage, media type alias, undefined kind-removal
migration) and are amended into ADR-001 (Amendments section). Director decision:
no round 2 for ADR-001 - the amendments are additive specifications, not
structural changes; the two-round budget is preserved for ADR-002, which
inherits two hard requirements (op idempotency by UUIDv7 id, custody event
ordering) from this round's advisories.

## D-011 (2026-07-06) - Human gate answers received

The human answered the four open gates in one line: (1) git identity
`Patrick Freeland <accounts@indigenousaccess.org>` CONFIRMED; (2) naming
CONFIRMED as-is (product Community Navigator, folder community-connector);
(3) license: "polyform" - the director selected **PolyForm Noncommercial
1.0.0** as the variant (the family's mainstream choice; permits free
noncommercial community use, forbids commercial exploitation). LICENSE.md
added from the canonical polyformproject.org text via codex grind.
**Remaining one-liner for the human: confirm Noncommercial vs another
PolyForm variant (Internal Use, Small Business).** (4) Codex sandbox:
"figure it out" - director keeps the danger-full-access bypass with D-008
mitigations; fixing the elevated sandbox would mean risky surgery on the
desktop app's codex config for marginal benefit on blueprint-constrained
tasks.

## D-012 (2026-07-06) - Design brief round 1: REDESIGN verdict, directed revision

Codex review returned 10 blocking objections and verdict REDESIGN on
docs/design/DESIGN_BRIEF.md. Director ruling: the aesthetic direction stands;
the failures are implementation overclaims (3d-force-graph instancing,
uniform-only animation), an under-specified 5MB budget, a real theming
contradiction (colors-unmangled vs contrast enforcement - resolved: intent
leads, legibility wins), and a11y gaps (resolved: parallel DOM is the primary
equivalent interface). Revision applied by codex grind per
.codex/task-brief-revision.md; a Phase 3 rendering spike is now checklist
item 1. Critique round 2 DEFERRED to Phase 3 start, when spike results exist
to critique against - the two-round budget is not spent, it is parked.

## D-013 (2026-07-06) - ADR-002 accepted after two adversarial rounds

Round 1 (REDESIGN): 8 blockers, all amended (see ADR-002 Amendments). Round 2
(ACCEPT-WITH-AMENDMENTS): confirmed 6/8 resolved; director fixed the final
three findings directly - per-field sort_key comparison (quarantine admission
order can never violate LWW canonical order), typed snapshot checksum
recovery, and an explicit TierSet authorization predicate (governance
any-within-policy OR owner strict-tighten) that removes the contradiction
with the permission spec. Round budget spent; ADR-002 is accepted.

## D-014 (2026-07-06) - Routing lesson: effort-match grind tasks

The design-brief revision on grind (gpt-5.4-mini, low effort) applied 8
substantive rulings in only 24 inserted lines, leaving verified gaps
("unmangled" phrasing survived, WCAG citations missing). Rule adopted: grind
at low effort is for truly mechanical transforms; multi-ruling document
revisions get a stronger model or higher effort (pass 2 ran gpt-5.5 at
medium via -m override). Director spot-check greps after every in-place
revision are now standard (they caught this).

## D-015 (2026-07-06) - ADR-003 accepted after two adversarial rounds

Round 1 (REDESIGN): 5 blockers - unfiltered validation reports, unbounded
export options, forgeable viewer contexts, leaking error payloads, and a
factually wrong one-crate cdylib/rlib claim. Amended: viewer-scoped reports,
narrow-only exports, honest v0 trust scope with a declared Phase 5
session-identity dependency, hidden-equals-absent error semantics, and a
cn-api facade crate. Round 2 (ACCEPT-WITH-AMENDMENTS): caught that category
COUNTS still leak (revoked - no counts ever for non-governance viewers),
entity_detail needed the detail-equals-projection rule, and the crate claims
had to be true in the scaffold, not just the prose - cn-api crate added,
cn-wasm made cdylib+rlib depending on cn-api, workspace re-verified green.
Round budget spent; ADR-003 accepted. With this, all three Phase 1 ADRs are
accepted and every acceptance criterion of Phase 1 is met.

## D-016 (2026-07-06) - Phase 2 closing review: 5 blockers accepted, 1 rejected

Codex review (gpt-5.5 high, whole core) returned FIXES-REQUIRED: 6 blocking,
4 advisory. Director rulings: B1-B5 CONFIRMED and fixed (export gate moved
into cn-perm per I2; submit now reports quarantine truthfully; hidden vs
absent made indistinguishable in submit outcomes; substring redaction
replaced with structural subject-based filtering; snapshots now round-trip
field clocks and seen-set - a real convergence bug). All four advisories
accepted (log version rejection, snapshot discard warnings channel, HLC
counter rollover, cn-api module split). B6 (cn-sync unimplemented) REJECTED
for Phase 2: the phase plan assigns the SyncTransport trait and local
adapter to Phase 5; ADR-002 A-B8 defines the contract it must meet then.
Fixes applied via codex grind from .codex/task-core-fixes.md; director
re-verified and committed.

## D-017 (2026-07-06) - Renderer decided on measured evidence (ADR-004)

The rendering spike ran on the reference Iris Xe in headed Chrome 149,
driven via playwright-cli (the claude-in-chrome extension was not
connected; playwright headed was the second automation path and worked).
Numbers in ADR-004: instanced 33.5 avg FPS at 5 draw calls (passes);
stock 3d-force-graph 26.1 and three-forcegraph 22.0 at ~10k draw calls
(fail). Decision: custom instanced Three layer owns all rendering; one
merged-LineSegments edge system; graph libraries demoted to spike-only
devDependencies. Headless control run agreed on ordering. The spike
harness (app/spike) is the standing regression benchmark.

## D-018 (2026-07-06) - Design brief round 2: accepted with amendments; budget spent

Round 2 (codex review, with ADR-004 evidence in hand) found 5 ADR
contradictions, 1 genuinely unresolved round-1 item (emitParticle
reintroduced the library edge API), and 7 checklist corrections - all
accepted and applied via directed grind revision (gpt-5.5 medium per
D-014; an initial wrong-tier launch at mini/low was caught and killed
within a minute). Director grep-verified the revision. The brief's
two-round budget is now spent; it is Phase 3's working document, revised
further only by ADR-anchored evidence.

## D-019 (2026-07-06) - Session 2 decision interview conducted through Part 5

The human ran the docs/SESSION_2_LAUNCH.md interview live (Parts 1-5
answered; entries D-020..D-038 below). The human ended the interview at
Part 5; Parts 6-8 fall to their published defaults: Q6.1 CSV-first
ingestor with column mapping, Q6.2 always-queue duplicates, Q7.1 full
autonomy with the ARCHITECTURE-redesign parking rule, Q7.2 current spend
acceptable (human: "there is plenty of context and usage"), Q7.3 the
8:00 AM safety cron and usage-failover directives stand, Q7.4 PolyForm
Noncommercial stands. Per protocol, pure defaults get no separate entries.

## D-020 (2026-07-06) - First deployment: ATNI committee pilot (Q1.1)

The first real community is an ATNI committee pilot (the Climate
Resilience Committee per D-022), not a CPF-RCN returning demo. Confirms
committee-first Phase 4 ordering; CPF-RCN becomes a later migration
target (D-031).

## D-021 (2026-07-06) - Hero workflows: explore + need-to-solution routing (Q1.2)

Both together: exploring the network (viz-first) AND need-to-solution
routing ("who here can help with X"). Routing UI is now explicit v0.1
scope in S3-B. Personal profiles were not selected (see D-029).

## D-022 (2026-07-06) - The convention pilot arc and win definition (Q1.3)

Human's scenario, condensed: before the ATNI Annual Convention, a consent
email goes to the Climate Resilience Committee plus documented past
convention attendees (other committees also invited to opt in).
Participants complete an intake form (name, Tribe, org(s), specialties,
resource availability, contact/social links, plus the fields designed in
D-023). The convention attendee list is used for outreach. The full
participant + committee graph is shown at the general assembly;
participants explore support pathways during the committee meeting;
feedback is captured to improve the system. WIN (near-verbatim): "they
see it work, they see themselves within it, they gain an understanding
of their connections, and then are able to visualize how this platform
enables them to share resources, information, and accelerate outreach."
Consequences: v0.1.0 has a real external date (the convention); the
intake form is the project's critical-path artifact; the pilot is
facilitator-run end to end.

## D-023 (2026-07-06) - Intake form design direction (director deliverable, Q1.3)

Principles adopted for the form + ATNI template (one artifact, two
views): (1) offers and needs draw from ONE controlled capability
taxonomy so need_met_by pathways compute directly; (2) edge-generating
questions (named collaborators, projects, committees, convenings)
outrank attribute questions; (3) response rate is protected via a
required ~5-minute core + optional depth section + facilitator-assisted
completion; (4) capacity level per offer and a contactability consent
(yes / through facilitator / no) are routing-critical fields; (5)
per-field visibility consent on the form maps to cn-perm circles at the
source; (6) never ask enumeration of traditional knowledge holdings -
only willingness to be contacted, at the most restricted tier. Full
field list delivered in-session; becomes the FORM docs deliverable.
All community-facing text requires human review before use.

## D-024 (2026-07-06) - Snapshot-first distribution (Q2.1)

The offline single-file snapshot is the primary v0.1 vehicle; the live
app remains the dev/build environment. S3-C snapshot acceptance and
Phase 6 targets follow.

## D-025 (2026-07-06) - Runs on the facilitator laptop (Q2.2)

Assembly and committee-meeting exploration run on the facilitator's
laptop (projector). Perf target stays the reference Iris Xe; no
mobile/touch scope in v0.1.

## D-026 (2026-07-06) - Backup: risk accepted (Q2.3)

Human answer: "Accept the risk for now." No remote, no bundles; the
remotes gate stays CLOSED. Single-machine total-loss remains risk #1 in
the register and is re-raised at every decision session. Do not act
autonomously on this.

## D-027 (2026-07-06) - Identity design deferred entirely (Q3.1)

No identity mechanism is designed now (no claim codes, no passkeys).
Session D and the identity ADR leave the v0.1.0 plan and return with
v0.2 personal-mode planning.

## D-028 (2026-07-06) - Facilitator (and developer) role now; creator governance later (Q3.2)

Human answer (verbatim): "For the demo and through the ongoing
development, there will need to be a facilitator (and developer) role
for entry, modifications, but will transition eventually into group
creator with permissions requirements." Scope: a standing facilitator
role with entry/modification authority is added to cn-perm's role model
for the pilot and development era, designed to hand off to
group-creator-held governance with permission requirements later.
Permission-adjacent work = grind HIGH + adversarial review.

## D-029 (2026-07-06) - Personal mode is v0.2, after the pilot (Q3.3)

Phase 5 stops gating v0.1.0. v0.1.0 ships facilitator-managed data with
the viewer-switcher demonstrating permission filtering. Personal mode is
built time-boxed after a real committee has used facilitator mode.

## D-030 (2026-07-06) - Graph membership: form respondents only, QR joins (Q4.1a)

Human answer: "We will utilize form respondents only, but QR codes to
the form and in convention packets will allow attendees to join."
Rulings: the intake form is the individual consent instrument; the
attendee list is outreach-only and is never rendered; QR-code joins
during the convention are consented joins, which creates a REQUIREMENT
for fast idempotent re-ingest + snapshot rebuild so same-day joiners
appear by the committee meeting (S4-A).

## D-031 (2026-07-06) - CPF-RCN migration recipe written in Phase 4 (Q4.2)

Session E stays: docs-only recipe (export, scrub, tier assignment, FPIC
checkpoints). Execution remains human-gated; the session never reads
red data.

## D-032 (2026-07-06) - TSDF codes are the primary tier language in the UI (Q4.3)

Against the plain-language recommendation - deliberate human choice, do
not "fix" later. T0-T3 codes are the visible UI language; plain-language
equivalents appear secondarily (tooltips/expansions). Aligns the UI with
the TSDF standard (C:\dev\TieredSovereignDataFramework).

## D-033 (2026-07-06) - Provenance visibility: one-line for members, full chain for governance (Q4.4)

Members see "added by X from Y (date)"; governance sees the full
IEEE-2890-style custody chain. Detail-panel design input for S3-B.

## D-034 (2026-07-06) - Demo tiering: everything T1; ATNI Climate is the tier authority (Q4.1b/c)

Human answer (verbatim): "For this working demo, all entries and outputs
are considered Tier 1 (ATNI Climate assigns Tiers), but after feedback
and continued development, tier enforcement will be better developed."
Rulings: all pilot entries and outputs enter at T1; the tier-assignment
authority is ATNI Climate (the Climate Resilience Committee); per-field
tier-assignment UX and richer governance tooling are post-pilot work.
The collective FPIC checkpoint default stands: a recorded committee
approval of the activity before any real ingestion runs.

## D-035 (2026-07-06) - Accessibility deferred to post-pilot refocus (Q5.1)

Human: accessibility "is not a concern just yet, but after the feedback
we will refocus on this aspect." Session A leaves the v0.1.0 critical
path. RETAINED (cheap now, expensive to retrofit): the font-scale token,
reduced-motion variants, basic keyboard navigation. DEFERRED: the
parallel-DOM primary equivalent interface and the WCAG 2.2 AA audit.
This entry records the R9 acceptance-criterion deferral required by the
phase-gate rule (CLAUDE.md phase plan).

## D-036 (2026-07-06) - Facilitator wizard; template authoring stays JSON (Q5.2)

Group creation is a facilitator-led wizard from existing templates;
authoring NEW community types remains a JSON-file task until two real
communities have shipped. Session B scope set.

## D-037 (2026-07-06) - In-app story authoring is v0.1 scope (Q5.3)

Beyond the viewing-only recommendation - deliberate. The facilitator
composes stories (from intake-form story material) inside the app before
the convention. S3-C grows: authoring UI (create/edit/order steps
referencing entities) plus viewing.

## D-038 (2026-07-06) - Aesthetic check deferred to a dedicated design session (Q5.4)

Human: "We'll spend more time on the visual display later, and bring in
some focused agents and claude design." Hearthlight stands provisionally;
a human-present DESIGN sitting (focused design agents + cultural palette
review) is scheduled into the plan before convention polish.

## D-039 (2026-07-06) - Execution Plan v2 adopted (provisional)

PROJECT_PLAN.md section 3 rewritten from the interview: v0.1.0 =
convention-pilot ready (facilitator-run, snapshot-first). Phase 5,
Session D (identity), Session A (accessibility), and Session F's
governance-tooling remainder move past v0.1.0 to v0.2. S3-B gains the
routing UI; S3-C gains story authoring; a FORM docs deliverable (intake
form + ATNI template + consent email) joins the critical path; S4-A
gains fast re-ingest. PROVISIONAL: a plan-v3 sitting follows the
graph-networks research report (D-040); expect refinement, not reversal.

## D-040 (2026-07-06) - Post-interview directive: graph-networks deep research

Human directive: conduct deep research on graph databases beyond social
network analysis - 3D visualization, spatial reasoning, affinities,
resource pools, geographic mapping, what a graph of many Peoples could
illuminate, and real-world (non-social-media) innovations of the past
decade - then report comprehensively (findings, code and technical
structures, real-world usability). Report lands at
docs/research/graph-networks-report-2026-07-06.md. The next human
sitting shapes plan v3 with this knowledge, including possible
integrations: cap-assessor, TCR-policy-scanner, GeoBase,
engagement-database, and the TSDF ecosystem under C:\dev.

## D-041 (2026-07-06) - Integration plan via adversarial multi-model panel

Human directive: run an adversarial agent discussion (including Codex) over the
research report + codebase, make integration recommendations, and have an Opus
4.8 max-effort reviewer articulate the plan + technical specs. Executed as:
codebase surface map (Fable code-explorer) -> two opposed proposers MIN
(pilot-first) / MAX (spine integrator) -> two critics: Codex gpt-5.5 high
(session 019f3a59-91c9-7830-97f3-823a0c21069c) on engineering + a Fable skeptic
on sovereignty/delivery/decision-fidelity -> director synthesis -> Opus 4.8 at
MAX effort review. Deliverable: docs/design/integration-plan-2026-07-06.md.
Panel artifacts in .codex/ (gitignored): panel-codebase-map.md,
panel-proposal-{min,max}.md, panel-codex-critique.md.

Verdict: MIN is the base (hits the convention date, hugs D-019..D-040, maps to
the D-022 win); MAX's general-DTO ambition is deferred, its "don't foreclose the
spine" warning kept as a design note. Six blocking findings surfaced and are
resolved in the plan. Three are decision-relevant enough to flag here:

1. KEYSTONE - sovereignty and licensing are ONE fix. Both proposers seeded the
   capability vocabulary from Open Eligibility (a US settler taxonomy, CC BY-SA).
   Rejected: (a) it grants a vocabulary authority the human never delegated
   (D-034 reserves the analogous tier authority to ATNI Climate), and (b)
   CC BY-SA vs PolyForm Noncommercial is a license conflict. Both dissolve by
   adopting the HSDS taxonomy-agnostic STRUCTURE while ATNI Climate authors its
   own TERMS first (facilitator co-construction, Net-Map method, under FPIC).
   This REDIRECTS D-023's taxonomy direction. New human gate G1 (vocabulary
   authority) + G2 (any later Open Eligibility mapping isolated as separately-
   licensed third-party data).
2. Routing is NOT "UI-only" (Codex): PathRequest needs concrete endpoints and
   search returns attribute hits, not "who can help." The plan adds a term+asker
   -> candidate-paths contract, and - corrected by Opus - the contactability-
   consent gate (D-023 principle 4) is STRUCTURAL in cn-graph/cn-api candidate
   resolution, not a UI rule (I2 / ADR-001 A-B1 forbid app-layer permission
   logic). No "need-met/closed" state is ever built (caution #2).
3. The assembly comprehension layer (flat/list reveal projection, "how to read
   this" primer, facilitator reveal script, seeded+tested Stories) is promoted
   to a named CRITICAL-PATH deliverable with a rehearsal acceptance check - the
   graph-literacy caution is measured in a community like ATNI's, and the D-022
   win depends on the reveal landing.

Opus 4.8 (max) caught three must-fix errors in the first synthesis - an
unimplementable idempotency mechanism (no home for a source-id map in group
state; corrected to deterministic UUIDv5 identity for entities, edges, AND
custody), a nonexistent file reference (cn-perm/session.rs -> cn-perm/viewer.rs
+ cn-api/session.rs), and the misplaced consent gate - all fixed before commit.
The plan is PROVISIONAL input to plan v3 (the D-040 sitting); it changes no
accepted ADR and reserves G1/G2 to the human.

## D-042 (2026-07-17) - One-shot session directive: gpt-5.6-sol repin, Codex full read/write

Human directive at session start (verbatim intent): pick the project up, give
Codex full read/write permissions, run the session on gpt-5.6-sol models, and
strategize a one-shot completion of the gate-blind scope with Codex as a
continuous adversarial thought partner and Claude workflow subagents for
parallel work. Actions taken: grind and review profiles repinned from
gpt-5.4-mini / gpt-5.5 to gpt-5.6-sol (effort low / high; the adversary
profile already ran gpt-5.6-sol); all three profiles set danger-full-access
(the Windows ACL sandbox backend fails on C:\dev and workspace-write silently
downgrades to read-only under codex exec - the behavioral guardrail is
C:\dev\AGENTS.md). Codex CLI 0.144.0 cannot enumerate model variants, so the
confirmed family base id is used for all roles per Codex's own capability
interview (.codex/capability-interview.md). docs/ENVIRONMENT.md and
docs/CODEX_GUIDE.md section 2.4 updated to match. Human gates are NOT
affected: G-RAT, G-DATE, G1, G2, D-023 review, no-remotes, no-real-data all
stand; the one-shot targets the gate-blind scope of PLAN_1.0.md Phases 0-4
plus non-gated Phase 5 items, parking everything gated.

## D-043 (2026-07-17) - P0.3 enforcement design: scoped check-all on pre-commit, no pre-push

Trigger: PLAN_1.0.md P0.3 requires the battery enforced by a local hook without
crossing the no-remotes gate (D-026). A pre-push hook never fires with no
remote, and adding any remote is a human gate, so pre-commit is the only local
enforcement point that actually executes.

Choice: expand scripts/hooks/pre-commit to run the staged PII scan always and
scripts/check-all.ps1 -Staged -Quiet when staged paths touch code (core/, app/,
schemas/, fixtures/, scripts/). The -Staged trigger map keeps per-commit cost
proportional (measured on the reference laptop, warm caches): docs-only ~1s
(PII only), app-only ~10-15s, core-touching ~3min (fmt, clippy, test,
wasm-pack, smoke, snapshot). The PLAN_1.0.md impracticality threshold (~3min
for an app-only docs-adjacent change) is not met - app-only commits are well
under it - so the fast-subset fallback (fmt/typecheck/PII only, with full
check-all as a pre-commit-series gate) was NOT taken. Core commits pay ~3min,
which is judged proportional for permission-adjacent work. Full check-all
(everything, warm) is ~3.5-4min and remains the phase-exit / pre-commit-series
standard in AGENTS.md.

Verified: a deliberately failing vitest blocked a commit through the hook
(check-all reported app-test FAIL, exit 1, HEAD unchanged); the breakage was
then reverted. Rejected: pre-push (never fires locally), unconditional full
battery per commit (breaks atomic-commit cadence for docs lanes).

## D-044 (2026-07-17) - One-shot execution rulings from the adversarial strategy round

The session strategy went through a Codex gpt-5.6-sol adversarial round
(review at C:\dev\_reviews\community-connector\2026-07-17_one-shot-strategy.md,
verdict CONDITIONAL NO-GO). Director accepted the findings; the rulings:

1. **Status model.** Gate-blind units land as "implemented and verified -
   phase exit PARKED on human items." No "Phase CLOSED" or milestone-exit
   claim is made this session; M0 and later exits await P0.1, P0.6, and the
   human-only acceptances. Honest end state: separable gate-blind foundations
   landed; ratification-dependent critical path (importer, routing, FORM,
   rehearsal) remains designed but nontrivial.
2. **DP-1: generic importer contract = NO.** The G1/Q-B blocking-map rows in
   the untracked route map lose to the explicit G-RAT park (routing semantics
   and importer contract) and the accepted D-041 anti-general-DTO ruling.
   This session lands only: a zero-semantics `cn` router (help, typed exit
   codes, unknown-subcommand failure tests), `validate`/`export` adapters
   over already-ratified core behavior, and parked stubs for `ingest`
   (names G-RAT) and `snapshot` (names the Phase 2 envelope dependency).
   No generic column-mapping schema, no mapping fixtures.
3. **Wave-build concurrency model.** Parallel lanes edit disjoint file sets
   and NEVER stage or commit; a single serial integrator owns the git index,
   app/src/main.ts and shell mounting, package.json/package-lock.json,
   scripts/check-all wiring, generated wasm pkg, and DECISIONS/HANDOFF edits,
   committing unit-by-unit at wave barriers after one quiescent full
   check-all. Two check-all instances never run concurrently.
4. **Mechanics/language split (D-023/G1 protection).** All user-visible
   instructional, consent, or tier prose (primer, story seeds, reveal
   script, wizard/form help) is marked "DRAFT - PENDING HUMAN REVIEW
   (D-023)" and non-deployable; generic UI chrome is not gated prose.
   Fixture vocabulary stays inside the two existing synthetic domains; no
   plausible ATNI capability terms anywhere (G1).
5. **Snapshot discipline.** A byte ledger with a 4.2MB soft-ceiling headroom
   target is recorded after every size-relevant unit; one fixture per
   snapshot artifact; troika-three-text lands with a repo-bundled
   OFL-licensed font and an offline-render proof before broad UI work.
6. **Browser gate.** Automated snapshot acceptance uses a repo-local
   headless playwright test asserting non-zero projected entities, zero
   external requests, zero console/page errors, and no above-scope values
   in the serialized HTML. D-017's headed Chrome remains for measurement
   and human visual acceptance; it never gates check-all.
7. **Serialization compat.** GroupRole::Facilitator ships with membership
   round-trip and unknown-role loud-rejection tests (I7-adjacent).

## D-045 (2026-07-17) - Facilitator adversarial round: findings, fixes, rulings

The mandatory adversarial Codex round on the Lane C permission diff (D-028;
gpt-5.6-sol review profile, 2026-07-17 session, receipt in the session log;
verdict BLOCKING FINDINGS: yes) returned three blockers and one advisory.
Integrator rulings at Barrier 1:

1. **StoryUpdate blind overwrite (blocker - FIXED).** The loosened rule
   checked only the submitter role, so a facilitator could overwrite a
   hidden story by guessing its id. Fixed target-aware in
   cn-perm/src/authz.rs (authorize_story_update): governance unrestricted;
   facilitators require an existing, visible target (target_missing /
   target_hidden); role is checked before the target so non-facilitators
   learn nothing. Cells added to authority_matrix.rs; doc updated.
2. **"facilitator" enum value under the 0.1.0 line (blocker - RULED, no
   code change).** Widening the persisted role value set without a minor
   bump would strand same-line readers. Ruling: the 0.1 line is unreleased
   with zero external readers (no remotes, D-026; schemas authored this
   wave document the tree as-is, facilitator included), and the ratified
   blueprint pinned "op-log major stays". The widening is absorbed into
   the unreleased 0.1.0 definition. Standing rule going forward: once any
   persisted format has left this machine, enum value-set widenings bump
   the compatibility line (minor while major is 0) with reader tests.
3. **Five-class no-leak property scope (blocker - PARTIALLY ACCEPTED,
   hardened).** Search/path/export all consume the Projection
   (GraphIndex::build(&Projection); exports serialize it), so the
   project() property covers the sole read root; surface-specific leak
   paths do not exist by construction. Accepted hardening: the property
   now also asserts report redaction per generated viewer class
   (governance exact; everyone else zeroed counts, no invisible-subject
   warnings, only own finding-stripped quarantine stubs).
4. **CRC32 fingerprint cache collision (advisory - FIXED).** Codex
   produced a concrete member/facilitator canonical-input pair colliding
   at the 32-bit viewer_fingerprint for any shared template suffix
   (verified independently), voiding P3.3's "never cross-serve" claim.
   Fix: cn-api GroupSession caches now key on new
   cn_perm::viewer_cache_key (the canonical authorization context,
   collision-free, in-memory only, never serialized); the exported
   Projection.viewer_fingerprint field and its 8-hex schema pattern are
   unchanged (ADR-003 untouched). Regression test pins the collision pair
   (cache_key_survives_fingerprint_collision). Rejected: raw-context
   cache keys in exports (leaks grant ids); new crypto-hash dependency
   (heavier than needed for an in-memory key).

## D-046 (2026-07-17) - Wave 1 review sweep: rulings on blockers and advisories

The barrier wave review (Codex gpt-5.6-sol review profile, session
019f7399-fdc7-75d3-b629-422b8eee6147, verdict BLOCKING FINDINGS: yes over
5495036..HEAD) returned two blockers and four advisories. Integrator rulings:

1. **Write-outcome existence oracle via story steps (blocker - RULED
   pre-existing, PARKED as priority Wave 2 design work).** A submitter can
   distinguish existing-but-hidden entity ids from nonexistent ids through
   the Applied vs Quarantined submit outcome of ops that reference
   entities. Verified scope: this is NOT a facilitator regression - the
   member-open StoryCreate (and EdgeCreate) apply paths have carried the
   identical oracle since Phase 2 (fold.rs quarantines missing step
   entities for every member), so a StoryUpdate-only patch is ineffective
   (facilitators are active members and could probe via StoryCreate). An
   effective fix uniformly adds reference-visibility authorization to
   member-open create/update ops in cn-perm - a change to ratified
   ADR-002 apply semantics that is itself permission-adjacent and needs
   its own blueprint plus mandatory adversarial round. Mitigations
   meanwhile: ids are UUIDv7 (blind guessing infeasible; the oracle only
   confirms possession of an already-known id), read surfaces stay
   projection-filtered, and the pilot threat model is facilitator-run
   devices. Owner: Wave 2 permission lane.
2. **Snapshot-envelope schema cannot bind scope to payload (blocker -
   FIXED as a documentation boundary).** Correct observation: JSON Schema
   validation is structural and can never prove the export was computed
   for the declared viewer_scope. The schema over-claimed. Fixed by
   stating the I2 enforcement boundary explicitly in the schema
   descriptions: scope truth lives in the P2.3 generator (export obtained
   only via cn-api/cn-perm for the declared viewer; generator must refuse
   viewers whose projection exceeds group-member reach) and the D-044.6
   snapshot acceptance test (no above-scope values in the artifact).
   Schema validation was not and is not the permission gate (no instances
   exist until P2.3).
3. **Nested story/provenance schema_version not checked at apply
   (advisory - PARKED).** Runtime readers enforce the operation's
   schema_version; embedded record versions ride the op line. Wave 2:
   either a reader-side nested-version check or an explicit documented
   rule that op.schema_version governs embedded records.
4. **Snapshot worker outside the measured artifact (advisory - already
   tracked).** dist/worker-*.js (1.55MB) is externally referenced by the
   snapshot HTML; single-file inlining and honest budget accounting are
   the Phase 2 snapshot-envelope lane's existing work item (P2.3).
5. **Latin-only font subset (advisory - already documented).** Non-latin
   display names would hit troika's CDN fallback (console error + missing
   glyphs offline). Post-pilot font-coverage decision stands (Lane A
   record).
6. **validate-templates emits no machine-readable report (advisory -
   PARKED).** I12's machine-readable reports exist in the Rust core;
   extending the build-time validator with a JSON summary is queued for
   the Wave 2 schemas lane.

## D-047 (2026-07-18) - Wave 2 recovery rulings (post spend-limit interruption)

The Waves 0-2 build workflow was interrupted by an Anthropic monthly spend
limit at Barrier 2. HEAD (e43cfc1) is a green Barrier-1 checkpoint (Wave 0 +
Wave 1 committed); Wave 2 (legend/motion, search/detail/flat, snapshot
envelope, cn CLI router) is uncommitted in the working tree. Per the
usage-failover directive (CODEX_GUIDE section 6), recovery is offloaded to
Codex gpt-5.6-sol with Claude as director; offloaded commits are marked
[codex-offload]. Codex triage (C:\dev\_reviews\community-connector\
2026-07-18_wave2-triage.md) returned SALVAGE for all four lanes. Director
rulings:

1. **Operator/developer chrome is exempt from the D-044.4 / D-023 language
   quarantine.** The DRAFT-PENDING-HUMAN-REVIEW marker and G1 vocabulary
   quarantine govern text shown to pilot participants/community: the
   participant-facing app (detail-panel tier explanations, primer, story
   prose, wizard/entry-form help) and the pilot's outward documents (intake
   form, consent email). Developer/operator chrome - `cn` CLI --help/usage/
   parked-stub messages and build-tool diagnostics - is NOT community-facing
   and needs no marker. The always-visible detail-panel tier summary IS
   participant-facing and must carry the visible marker (triage finding 163).

2. **Detail-panel provenance/tier (P1.5, D-032/D-033).** The I2-respecting
   path is to extend cn-api::EntityDetail so cn-perm supplies the effective
   tier code (D-032) and a provenance one-liner, with full custody depth only
   when the viewer projection is governance (D-033); the app renders what it
   is given and never infers role. Bounded-scope park rule (deep-thinking
   ladder): if that core extension exceeds a few DTO fields + one cn-perm
   summary function + tests, PARK it - ship detail with attributes + owner
   indicator, remove the dead detail.provenance rendering, and record the
   one-liner as a small deferred follow-up. No dead UI, no I2 violation
   either way.

3. **A2b scoped-response correctness is BLOCKING.** Search and detail
   actions/reducers must carry group+viewer+session request identity and
   reject stale or out-of-scope responses in the state machine (request-
   identity matching, not app-layer permission logic - I2-safe). Closes the
   old-scope-data-into-new-session risk the triage flagged.

4. **B2 snapshot completion is required to reach green** (its partial wiring
   currently fails build:snapshot loudly on empty anonymous projections):
   a deterministic public-layer synthetic research-network fixture generated,
   reviewed, and committed (never silently rewritten at build time); pkg-node
   rebuilt with a freshness check; MISSING WORKER INLINING IS FATAL (D-046
   self-contained single file); per-artifact (snapshot.*.html) size gate;
   envelope format version named independently of the export version; the
   double projection pass removed; and a real snapshot BOOT READER in main.ts
   with unknown-major rejection (I7 reader half). ACCEPTANCE GATE: no snapshot
   artifact is committed until a no-leak test that inspects the serialized
   HTML for above-scope sentinel values passes (P2.5).

## D-048 (2026-07-18) - Snapshot data pipeline (P2.3-P2.5) decoupled and parked

After three consecutive Codex exec sessions exited on an internal step cap
before finishing B2, the director decoupled the snapshot DATA pipeline from
the rest of Wave 2. State at the decision: check-all was 10/11 green - only
app-snapshot failed, because the committed fixtures are all group-visibility
and the embed produces an empty anonymous projection, and because the
snapshot still emits a ~1.57MB EXTERNAL worker chunk (a file:// single-file
snapshot cannot load an external worker, D-046).

Root architectural finding (director): main.ts hydrates the initial snapshot
render from the embedded envelope with no wasm, but search and detail still
call cn-api through the Worker at runtime, so an offline self-contained
snapshot needs an in-process (main-thread) WasmTransport in snapshot mode -
the WasmClient already accepts a WasmTransport, so this is additive, but it
was not finished. This is the correct v0.1 design and is the parked follow-up.

Decision: the P2.3 embed plugin is committed but GATED behind
CN_EMBED_SNAPSHOT=1 (app/vite.config.ts); the default snapshot build produces
a valid single-file shell so check-all stays green. The snapshot boot reader
(app/src/state/snapshot.ts, with I7 unknown-major rejection) and its unit
tests are committed and active. PARKED as a scoped follow-up (P2.3 completion):
(1) main-thread WasmTransport for snapshot mode so the artifact has no
external worker; (2) wire the --public-layer generator into build:snapshot
into an isolated non-tracked dir; (3) per-artifact size gate in
check-size.mjs; (4) the no-leak acceptance test (D-047.4) asserting no
above-scope sentinel appears in the serialized HTML; then flip
CN_EMBED_SNAPSHOT on by default. Everything else in Wave 2 (explore surface,
detail provenance/tier core extension, CLI router) is complete and committed.
This changes no accepted ADR and crosses no human gate.

## D-049 (2026-07-18) - R2 adversarial round completed LATE with BLOCK; fixes queued

Reconciliation during /trueup: the mandatory permission-adjacent adversarial
round on the R2 EntityDetail change (D-044; committed 5e9c176 as
[unreviewed-by-codex] because the round appeared to have exited early) actually
COMPLETED and wrote its verdict late, at C:\dev\_reviews\community-connector\
2026-07-18_r2-entitydetail-adversarial.md. Verdict: BLOCK. The record is
corrected here: the round did run; the R2 commit stands but carries known,
reviewer-confirmed defects to fix as the next session's first unit. The tree
is green and safe to sit on (the defects are a display mislabel and a
pre-existing boundary issue, not a data leak).

Findings and disposition:
- **CONFIRMED SAFE (custody gate).** The reviewer's own trace agrees: the only
  constructor of full custody depth is the `is_governance` arm; anonymous,
  member, facilitator-only, self, and trust-only viewers receive only the
  one-line summary (D-033 holds). project() and the five-class no-leak property
  are unchanged - no regression.
- **BLOCK-1 (tier under-report, D-032 correctness).** `entity_detail_metadata`
  returns raw `entity.tier`; a viewer's projected attributes can carry a higher
  effective tier (fixture: T0 entity + T2 attribute override visible to a
  member, cn-perm/tests/blueprint.rs:655-712), so the detail can show
  "tier":"T0" beside viewer-visible T2 data. FIX (cn-perm, I2-clean): report the
  effective tier as the max of entity.tier and the effective tier of exactly the
  PROJECTED (viewer-visible) attribute set - never over raw attributes (which
  would reveal hidden higher-tier attributes).
- **BLOCK-2 (I2, pre-existing).** `own_settings` in cn-api/src/lib.rs:385-421
  decides tier/visibility disclosure by owner_is_viewer and computes effective
  tier in the API layer. It predates R2 but is on the detail path. FIX: relocate
  the disclosure/effective-tier decision into cn-perm; cn-api carries decided
  values only.
- **HIGH-3 (test gap).** The custody/tier tests cover only member+governance.
  FIX: extend to anonymous, facilitator-only, self, trusted/non-member,
  inactive-governance, and dual-role, plus an end-to-end cn-api entity_detail
  test with a non-empty custody chain (governance sees the exact chain; every
  non-governance class omits it).
- **LOW-4 (one-line shape).** actor_summary/origin_summary
  (cn-perm/src/projection.rs:99-112) do not normalize control whitespace, so a
  newline in an agent id or ingested source breaks the D-033 one-line shape. FIX:
  normalize whitespace in the summary.

These are gate-blind (pure cn-perm/cn-api). They are the next session's first
unit and must get a fresh adversarial round after the fix. Operational note:
Codex `exec` output files can land LATE (after the process appears done) - verify
by re-checking the output path before concluding a round failed ([[codex-exec-early-exit]]).

## D-050 (2026-07-24) - Pilot calendar set: convention 2026-09-14, August internal pilots

Human answered G-DATE / Q-A in the gate-grill session. The ATNI Annual
Convention is 2026-09-14. Early September is the soft deadline for
pre-convention consent; most joins are expected at the convention itself.
New commitment: internal pilots with several trusted groups run BEFORE the
convention (August), building on CPF-RCN demo experience, to accelerate
development. Consequences: the intake -> review -> ingest -> render pipeline
must be usable by mid-to-late August; the D-030/D-034 consent process
(form-based individual consent, outside-repo staging, T1 tiering) applies to
the internal pilots too; the recorded ATNI Climate collective checkpoint must
precede the FIRST internal-pilot ingestion, not merely the convention.

## D-051 (2026-07-24) - G1 answered: ATNI authors the vocabulary, post-stability

Human ruling: ATNI Climate authors the capability vocabulary in its own words
(option a), but language work is sequenced after the system is fully
functional and stable. Until then the backend/schema layer uses standard
developer language over the empty HSDS-shaped structure (the D-044.4
mechanics/language split). A Claude-Design pass over front-facing interface
text and language corrections is planned for the later stage.

## D-052 (2026-07-24) - G-RAT answered: v0.1.0 ratified as the convention-arc finish line

v0.1.0 is ratified as the committed finish line for the convention arc, with a
sharpened acceptance bar: ready for ACTUAL USAGE across the real entity kinds
(persons, organizations, places, skills, needs) at pilot scale (~150 expected,
300 max signups), not demo-grade. Ratification of the fuller 1.0 line (Phases
6-9) is deferred to a post-convention retrospective, when pilot evidence
exists.

## D-053 (2026-07-24) - Intake architecture: QR -> static form -> sealed-envelope relay -> facilitator review queue

Q-B is answered by replacing the form-platform question entirely. Rulings, all
made by the human in the gate-grill session:

- No external form platform (no Google, no Microsoft). Direct in-app data
  entry is an initial feature; P3.5/P3.6 are the intake pipeline, not UI
  polish. CSV ingestion becomes a secondary path for structured sources.
- No auth in v0.1.0 or v1.0 (future possibility only).
- Facilitator-review staging: ALL submissions (in-app or remote) land as
  pending and enter the graph only on facilitator approval (the
  error/inconsistency gate; also the abuse gate that makes an unauthenticated
  endpoint tolerable).
- Remote intake flow (sealed-envelope relay): attendee scans QR -> static
  intake form hosted on GitHub Pages (interface only; no secrets; no readable
  data ever transits or rests there) -> the browser encrypts the payload to
  the facilitator's public key before it leaves the phone (libsodium sealed
  box; the private key exists only on the pilot PC) -> the ciphertext POSTs to
  a minimal Cloudflare Workers + KV relay whose only job is to store blobs it
  cannot read -> the pilot PC pulls, decrypts locally, and stages into the
  facilitator review queue; relay storage is wiped after pull. The graph
  itself never listens on the network; the pilot PC may be internet-connected
  (pull-based).
- Gates opened BY THE HUMAN: public git remote
  https://github.com/atniclimate/community-connector (created by the human
  2026-07-24, empty, public); hosting vendor Cloudflare Workers on the human's
  existing account; spend for this demo approved. The R5 "not networked"
  stance is amended for this one intake path; a localized (offline/hotspot)
  intake system is the later stretch goal.
- Deploy only after community-connector is stable (human directive).

## D-054 (2026-07-24) - Q-C answered: PolyForm Noncommercial 1.0.0

The license is PolyForm Noncommercial 1.0.0, committed to the repo before the
first push to the public remote.

## D-055 (2026-07-24) - P0.6 answered: track the three root docs after a pre-publish sweep

PLAN_1.0.md, MANIFEST.md, and DEPENDENCIES.md will be tracked, contingent on a
pre-publish review pass. Because the remote is public, the first push waits on
a full-repo sweep for anything unsuitable for world-readability (machine
paths, CPF-RCN remediation references, convention logistics, community-facing
text pending D-023 review). Borderline content moves to a gitignored
_private/ and is reported. Until the sweep unit runs, the three docs stay
uncommitted.

## D-056 (2026-07-24) - Look-back reconciliation: ADR-005 required, route resequenced for the August window

A two-agent look-back (doc-staleness sweep + invariant/risk assessment) ran
after the D-050..D-055 rulings; check-all was re-verified 11/11 green the same
day. The rulings below are director-level (phase-internal sequencing, ADR and
test strategy), autonomous per CLAUDE.md.

1. **ADR-005 is required.** D-053 trips the stance-change rule (R5 amendment, a
   new staging store, new persisted formats). One consolidated ADR-005 "Remote
   intake: sealed-envelope relay and facilitator pending-review queue" plus one
   adversarial round covers: R5 amendment scoping (intake is INGEST, not sync -
   the puller routes through cn-ingest concepts, never the SyncTransport seam;
   the graph never listens; the relay holds ciphertext only); sealed-envelope
   payload format + versioning and key pinning/rotation (I7); pending-queue
   placement outside the op log, persisted versioned format, durable-write
   BEFORE relay-wipe; the intake provenance envelope (I6: actor = intake
   tooling, responsible_human = facilitator, capture timestamp, form version,
   relay receipt id, client-generated submission UUID for dedup); threat model
   for the unauthenticated endpoint (payload size caps, rate limits, KV TTL).
2. **Ownership-at-approval default.** Approved remote submissions land as
   UNOWNED, facilitator-created entities (authority-matrix-clean). Owner-binding
   a record to its submitter is deferred; doing it later is an authority-matrix
   change and triggers its own adversarial round.
3. **Resequencing.** Internal pilots run the normal app build on the pilot PC -
   they do NOT need the snapshot pipeline. Snapshot work (D-048) moves after the
   intake pipeline and targets the convention build. The P1.3 benchmark defers
   to September; Phase 4 slims to minimal P4.1 story authoring for the pilot
   window. Long-lead gate-openers start in parallel with the R2 fixes: the
   D-055 pre-publish sweep, intake-form/consent text drafted into D-023 human
   review, ADR-005 drafting, and the facilitator keygen ceremony design.
4. **Risk register (August window).** Facilitator private key is a single point
   of total loss (needs a keygen ceremony, offline key backup - key backup is
   not repo data, so no G-BACKUP collision - and key-fingerprint pinning in the
   puller). Pull-then-wipe requires durable queue writes first. The Pages form
   is gate-coupled to the repo's publish preconditions, making the D-055 sweep
   pilot-critical-path. No-auth means dedup via payload UUID + facilitator
   near-duplicate surfacing. The least compressible items are human-path: D-023
   text review and the recorded collective checkpoint before the FIRST August
   ingestion.
5. **Doc reconciliation dispositions** (staleness sweep): CLAUDE.md gate notes +
   real-data process, AGENTS.md vocabulary timing, CODEX_GUIDE gate mirror,
   NEXT_SESSION.md, and cpf-rcn-migration-recipe.md are updated in this
   true-up. PROJECT_PLAN.md and pilot-form-and-template-2026-07-06.md carry
   reconciliation banners; their full revisions land with the D-055 sweep unit
   and the D-023 form-text draft respectively. PLAN_1.0.md, MANIFEST.md, and
   DEPENDENCIES.md get their updates inside the D-055 sweep unit (untracked
   until then). LAUNCH_PROMPT.md, the integration plan, research/analysis docs,
   and ADR critiques are historical records, left as-is. LICENSE.md is already
   tracked with the PolyForm Noncommercial 1.0.0 text, so the D-054
   precondition is satisfied on disk (re-verify at push time).

## D-057 (2026-07-24) - R2 EntityDetail fixes landed; fresh adversarial round PASS-WITH-NOTES

The D-049 queued fixes landed as one unit (commit 73b5656). BLOCK-1: the
detail tier is now the effective maximum over exactly the viewer-projected
attribute set, computed in cn-perm. BLOCK-2: own_settings was deleted from
cn-api; the owner-only visibility/tier disclosure and every other detail
decision live in cn-perm (DetailAttribute); cn-api is a pure carrier. HIGH-3:
deterministic custody+tier matrices across anonymous, member,
facilitator-only, self-owner, trusted non-member, plain non-member,
inactive-governance, active governance, and dual-role viewers - in cn-perm
unit tests, in the no-leak property (detail-surface extension), and
end-to-end through cn-api::entity_detail with a non-empty custody chain.
LOW-4: the provenance one-liner strips control characters AND U+2028/U+2029.

The mandatory fresh adversarial round (gpt-5.6-sol adversary profile,
2026-07-24, review artifact 2026-07-24_r2-entitydetail-fix-adversarial.md in
the out-of-repo review lane) returned PASS-WITH-NOTES: BLOCK-1 and BLOCK-2
explicitly CLOSED with counterexample attempts defeated; its residual notes
(missing trusted/dual-role deterministic tier witnesses, missing e2e trusted
custody case, U+2028/U+2029 surviving normalization) were closed inside the
same commit before it landed. The reviewer's performance note - the detail
path re-scans raw attributes per request - is ACCEPTED and documented in a
doc comment: detail is a single-entity interaction path, and recomputing in
cn-perm keeps it the sole authority. check-all 11/11 green before commit.
D-049 is fully discharged.

## D-058 (2026-07-24) - D-055 pre-publish sweep executed: dispositions, _private/ split, root docs tracked

A six-agent world-readability scan covered every tracked file plus the three
untracked root docs (42 findings), followed by revision passes. Dispositions
taken autonomously under the D-055 mechanism:

- **Edited in place:** username paths and the machine hostname removed from
  docs/ENVIRONMENT.md; the staging drive letter genericized in
  docs/cpf-rcn-migration-recipe.md; stray transcript markup removed from
  PLAN_1.0.md; the Codex session UUID redacted from cn-api/src/session.rs,
  cn-perm/tests/blueprint.rs, and the D-045 entry above (tracker metadata
  stays out of the repo); the stale tracked Playwright artifact
  app/test-results/.last-run.json untracked and gitignored.
- **Split:** DEPENDENCIES.md became a world-readable external-dependency
  audit; all machine-local operational content (backup manifest and robocopy
  recipe, absolute paths, predecessor PII-fencing specifics and sizes) moved
  losslessly to gitignored _private/DEPENDENCIES-local.md.
- **Revised and TRACKED per D-055:** PLAN_1.0.md (v1.1: G-RAT/G-DATE/G1/Q-B/
  Q-C/P0.6 marked RESOLVED with D-numbers, route resequenced per D-056.3,
  Phases 6-9 explicitly deferred to the post-convention retrospective, the
  P5.6 dedup ADR renumbered to ADR-006 since ADR-005 is remote intake);
  MANIFEST.md (re-dated 2026-07-24, reality-checked against HANDOFF.md);
  DEPENDENCIES.md (the split revision). docs/PROJECT_PLAN.md received its
  full D-056.5 revision (reconciled route, gate statuses, risk register).
- **Ruled acceptable as-is:** convention date and aggregate signup estimates
  in decision records; .gitignore defensive PII patterns; the license
  copyright name (required by PolyForm, D-054); the pilot-evidence template
  (correctly banner-marked DRAFT).

**The "sweep passed" push precondition is NOT yet fully satisfied.** Six
needs-human dispositions park on the human before the first push:

1. Maintainer emails in D-003/D-011 above and scripts/pii-allowlist.txt
   (the tenant address is the gratuitous one) - redact or accept.
2. Public disclosure that the predecessor repo holds real-partner PII with
   named directories (CLAUDE.md predecessor rules, docs/LAUNCH_PROMPT.md,
   docs/cpf-rcn-migration-recipe.md) - keep verbatim, generalize, or move
   specifics to _private/.
3. docs/THE_STORY.md makes audience-facing claims on behalf of ATNI with no
   review marker - confirm it is approved for the public repo.
4. docs/design/pilot-form-and-template-2026-07-06.md Parts A/C (draft form
   text and consent email, banner-marked DRAFT) - publish as marked draft or
   move to _private/ until D-023 review completes.
5. Workspace-layout paths in contract docs and archived handoffs (no
   usernames; operationally load-bearing) - recommended accept as-is.
6. The maintainer's mirror recipe does not exclude _private/, so the local
   mirror will carry it - confirm that is desired (recommended: yes, the
   content must not be lost; it simply must not be pushed).

## D-059 (2026-07-24) - Grill session rulings: pre-push dispositions, split stability bars, checkpoint posture, retention

The human ruled on every open blockage in a structured grill session. All ten
rulings below are the human's; recorded verbatim in effect.

1. **Redaction philosophy: targeted.** Redact only genuinely gratuitous
   exposure; keep workspace paths, historical records, and the git-identity
   email (public in commit metadata) as-is.
2. **Emails.** The machine-resolved tenant address is redacted from D-003 and
   dropped from scripts/pii-allowlist.txt; `accounts@indigenousaccess.org`
   stays.
3. **Exclusion list.** The enumerated predecessor-repo PII paths moved to
   gitignored `_private/PREDECESSOR-EXCLUSIONS.md`; CLAUDE.md keeps the
   binding rule with a pointer; LAUNCH_PROMPT.md carries an in-place
   bracketed redaction note; the migration recipe references the private
   list. The rule remains absolute regardless of the private file's
   availability.
4. **Workspace paths** in contract docs and archives: accepted as-is.
5. **THE_STORY.md: approved as-is.** Publishes with the first push; no
   further review step.
6. **Consent drafts stay public** as banner-marked drafts; the 2026-07-06
   Parts A/C carry a SUPERSEDED banner pointing at the 2026-07-24 package.
7. **Mirror carries `_private/`** (its only redundancy; never-pushed, not
   never-copied).
8. **Q-STABLE split bars.** PUSH bar: D-059 disposition edits committed +
   check-all green (no ADR-005 wait). DEPLOY bar (Pages form + Workers
   relay): ADR-005 ACCEPTED post-round + P3.5/P3.6 intake pipeline working +
   keygen ceremony executed + D-023 sign-off on form text. The human
   authorized sessions to execute the first push autonomously once the push
   bar is met.
9. **Collective checkpoint: timing unknown; default holds.** All engineering
   proceeds on synthetic data; the August internal pilots are explicitly
   CONDITIONAL on the recorded ATNI Climate approval existing - no session
   treats mid-August as committed.
10. **D-023 loop: bundled.** The human does the solo correctness pass soon
    (reviewer checklist in the package), which clears the text for build and
    synthetic-pilot use; the committee sees the reviewed text together with
    the checkpoint ask; community use waits for that moment.
11. **Rejected-record retention (amends ADR-005 D4 open question).** Full
    rejected records persist in the gitignored queue for the pilot window,
    then are purged in ONE RECORDED SWEEP - a mandatory dated checklist item
    at window close. Accepted implication: declined people's decrypted data
    persists on the pilot PC for the window.

With the disposition edits in this commit and check-all green, the D-053/
D-055 push preconditions are SATISFIED; the first push to
`atniclimate/community-connector` is authorized.

## D-060 (2026-07-24) - First push executed; the repo is public

With the D-059.8 push bar satisfied, the session executed the first push per
the human's explicit authorization: remote `origin` =
https://github.com/atniclimate/community-connector.git, `main` pushed and
tracking. Credential mechanics recorded for future sessions: the machine's
default git credential is the `indigenousaccess` account, which has no write
access to the atniclimate org repo; pushes authenticate as the `atniclimate`
account via the GitHub CLI keyring. Repo-local config now pins this
(`credential.helper = !gh auth git-credential` and
`credential.https://github.com.username = atniclimate`), so a plain
`git push` works from this clone. Standing consequence: every future commit
lands in public history on push - the pre-commit PII scan and the I1 prime
directive are now also the publication boundary. The deploy bar (Pages form
+ Workers relay) remains unmet and separate (D-059.8).

## D-061 (2026-07-24) - ADR-005 round 1: FAIL judged valid; ADR amended; round 2 required

Ladder rung 3 (research/design). The ADR-005 adversarial round ran per the
D-056.1 mandate (gpt-5.6-sol via the adversary wrapper; review at
`_reviews/community-connector/2026-07-24_adr-005-remote-intake.md`; target
HEAD b182fc1). Verdict: FAIL - five blockers (browser-bundle trust root,
Windows crash/approval transaction protocol, relay-to-queue idempotency,
consent/audit contract completeness, false key-custody claims), six majors,
two minors. Load-bearing claims verified against the actual files before
judgment: the keygen ceremony's printed+USB backups do contradict the ADR's
"only on the pilot PC"; the consent draft's active-confirmation requirement
had no corresponding payload field; pii-scan has no content-marker rule; the
approve-then-crash window between queue and op log is real. All five
blockers judged VALID; no finding was rejected outright.

The ADR was amended in place (same file, status now "round 1 FAIL judged
and amended; pending round 2"). Autonomous design choices made in the
amendment, logged here:

1. **D8 browser trust model (new):** deterministic dependency-closed build,
   deploy manifest, full-bundle SHA-256 pin held OFF-REPO on the pilot PC
   (extends the ceremony's key-only pin), deploy provenance via reviewed
   commits, CSP with single connect destination, no analytics. Residual
   split-view delivery risk stated and accepted for v0.1.0; absolute
   "nothing readable ever transits" claims withdrawn.
2. **D4 crash-state protocol:** temp-file + flush + atomic-rename primitive
   with checksums; single-instance lock; immutable payload record +
   append-history sidecar; approval as a write-ahead transaction with
   PREASSIGNED op ids reused on recovery (rides ADR-002 op-id dedup).
3. **Dedup split:** transport key (receipt_id, ciphertext_hash); semantic
   key (submission_id, payload_hash); same id + different hash = loud
   facilitator-disposed conflict, never a silent drop.
4. **Queue placement hardened:** repo-local gitignored staging REMOVED as
   an option; canonical root is the off-worktree facilitator ops dir; the
   puller refuses worktree/cloud-sync paths; at-rest preconditions (disk
   encryption, ACL, no indexing/sync) checked at startup. No queue backup:
   single-disk loss between pull and approval ACCEPTED over multiplying
   PII copies (revisitable in one line).
5. **Consent attestation added to the inner payload** (consent_text_digest,
   consent_affirmed, consent_affirmed_at) with all client fields recorded
   as source_asserted; provenance carries trust-status labels
   (source_asserted / relay_observed / facilitator_observed).
6. **D6 relay API contract:** server-generated 128-bit receipt ids; no
   public read/status oracle; verifier-model bearer credential
   (Worker-side hash only); NAT-safe rate sizing; total-capacity cap +
   billing ceiling; TTL policy bounds (>= 2x max pull interval) with pull
   service objectives (daily in August, hourly at convention) and
   POST-counter reconciliation; no-body/no-secret logging.
7. **Rotation rewritten drain-before-flip** with cache-horizon old-key
   retention, per the ceremony companion; emergency path assumes
   compromised-host artifacts are untrusted.
8. **PII tripwire claim downgraded** from enforcement to defense in depth;
   queue/secret marker rules with positive-fixture tests to be added to
   pii-scan; I1 process remains the boundary.
9. **Right-sizing:** signed-manifest second ceremony, encrypted queue
   backup, and any database/distributed queue REJECTED (options 9-11).

Parked for the human (no gate crossed autonomously): (a) the consent
draft's removal-semantics wording - "taken out of the network" vs the
append-only log - is now section 7 of the draft and the largest D-023
question; (b) the single-disk queue-loss acceptance (item 4) is flagged as
revisitable. Round 2 on the amended ADR is required before ACCEPTED; the
deploy bar (D-059.8) is unchanged.

## D-062 (2026-07-24) - ADR-005 round 2: FAIL judged valid; second amendment; round 3 required

Round 2 ran on the round-1-amended ADR (review at
`_reviews/community-connector/2026-07-24_adr-005-remote-intake-round2.md`;
target HEAD cc20638). Verdict: FAIL. Round-1 blockers 3/5 and findings
8/11/12/13 CLOSED; the rest partially closed with two NEW blockers, both in
mechanisms the round-1 amendment itself introduced, both verified against
the code before judgment: (1) the approval recovery assumed durable op-id
lookup/idempotent-append semantics cn-store does not have - authz.rs
reports an already-seen op as Applied and log.rs append_batch serializes
blindly, so recovery could duplicate audit-log lines; (2) the KV running
POST counter is unimplementable - no atomic increment under concurrent
POSTs, eventually consistent lists, and the pulled+deleted+expired equation
double-counts. Four majors: missing crash-recovery state table, no
executable bundle-measurement procedure (and ceremony contradictions), no
sidecar-payload binding + undecided approved-record closeout, unsound
rotation "proof". All judged valid; none rejected.

Second amendment, autonomous choices logged:

1. **Durable idempotent batch-append seam (D4):** additive cn-store API
   classifying preassigned op ids against the DURABLE LOG as
   absent/present-same-digest/present-conflicting-digest; whole-batch
   authorization before any append; append-absent-only + one fsync;
   conflicting digest = typed halt, nothing appended. approved_intent
   added to the versioned sidecar enum; batch failure returns the sidecar
   to pending with the failure in the decision history.
2. **Receipt ledger replaces the counter (D6):** per-receipt no-content KV
   ledger entries with TTL+horizon lifetime; reconciliation over DISJOINT
   states (staged / deleted-by-me / present / absent-not-mine ->
   expired-or-alert); hard-cap claim withdrawn - approximate cap + size
   cap + TTL + billing ceiling are the honest bounds.
3. **Relay admission cutoff (D3/D6):** the Worker validates the outer
   fingerprint against an allowlist; rotation removes the old fingerprint
   after drain, making "no old-key envelope can arrive" enforced, with a
   stale open tab getting a visible reload rejection; old key destroyed
   only after cutoff + one TTL + clean ledger reconciliation.
4. **Crash-state table (D4):** deterministic action for every on-disk
   state (temp/orphan/corrupt/binding-mismatch/approved_intent/lost
   history/degraded flush); corrupt/ quarantine retains relay copies.
5. **Sidecar-payload binding (D4)** (record_id + payload digest, verified
   on read) and **approved-record closeout DECIDED (D4/D5):** approved
   queue records are PURGED in the recorded window-close sweep (no
   archive branch - privacy first); audit residue = sweep manifest +
   provenance identifiers/digests; D5's "durable link" narrowed to an
   identifier that outlives its referent.
6. **Bundle measurement procedure (D8):** canonical manifest built
   LOCALLY from the reviewed commit (sorted normalized paths, per-file
   SHA-256+length, no self-hash), pinned off-origin; verification fetches
   every pinned path (same-origin, no redirects, identity encoding);
   extra-planted-file residual stated (unreferenced by verified
   HTML/CSP); unreachable-origin rule: pulls proceed with loud WARN, no
   new solicitation until verified.
7. **Ceremony companion amended in-commit** to match: active-key custody
   phrasing, bundle check in the puller gate, no-solicitation-on-skip,
   destruction timing per the cutoff rule.

The intake-pipeline blueprint (docs/blueprints/intake-pipeline.md, new
this session) aligned in the same commit: cn-store seam lands first in its
sequencing; sidecar binding and recovery classification in cn-ingest;
approval facade routes through the seam. Round 3 targets: crash points,
op-log persistence, concurrent POST/cap behavior, bundle verification
inputs, close-of-window audit continuity (the reviewer's stated round-3
scope). ADR-005 remains DRAFT until round 3 passes.

## D-063 (2026-07-24) - ADR-005 round 3: FAIL judged valid; third amendment - native durable owner

Round 3 (review at
`_reviews/community-connector/2026-07-24_adr-005-remote-intake-round3.md`,
target HEAD 3cb0ede) returned FAIL: three blockers, four majors, all
judged valid after verification (cn-store's log module is
`#[cfg(not(wasm32))]` - the round-2 seam was unreachable from the
browser/WASM approval path the blueprint proposed; recovery
re-authorization could mislabel durable ops; the two-write KV ledger had
no atomicity, observation window, or cutoff epoch).

Third amendment, the load-bearing design change first:

1. **Native durable owner (D4):** approval moves OUT of the browser
   entirely. The app is CREATE-ONLY (payload records, initial pending
   sidecars, decision files - unique names, never rewrites); native
   `cn intake apply` under the queue lock consumes decision files and owns
   all sidecar mutation, plan generation, the seam, fsync, and fold. The
   wizard becomes decide-in-app / apply-natively / reload. This supersedes
   the blueprint's FSA-rewrite + WASM-approval design (blueprint amended
   in-commit).
2. **Seam preflight (D4):** authorization and fold-acceptance preflighted
   on a shadow clone in batch order (fixes AttributeSet-after-EntityCreate
   denial; makes post-append quarantine impossible in the critical
   section).
3. **Intent-as-authorization-marker recovery (D4):** all-present-same-
   digest completes WITHOUT re-authorization; partial prefixes complete
   under the marker; digest conflict -> new terminal `failed` sidecar
   state (5-state enum) blocking retry until explicit facilitator
   disposition; sidecar-write failure halts the run (no livelock).
4. **Ledger (D6):** blob-then-ledger write order with ack-after-both;
   puller lists BOTH prefixes (orphans observable from either side);
   local-facts-first classification precedence makes the states a true
   partition; ledger TTL = blob TTL + consistency margin + one max pull
   interval (guaranteed observation window); receipt collision stated as
   negligible-by-randomness (no conditional-create claim); rotation
   cutoff completed only at config-propagation horizon + max request
   duration.
5. **Crash table (D4):** write-once review-begun marker distinguishes
   initial orphans from lost decision history; degraded namespace
   durability now RETAINS the relay copy (defers delete) instead of
   WARN-and-delete.
6. **Residual hard-cap claims swept** (three spots) to approximate-cap
   honesty.
7. **D8:** canonical manifest grammar (path rules, JSON sorted-keys LF
   UTF-8 no-BOM, hash over stored bytes, no self-hash), served manifest
   NON-AUTHORITATIVE, cache-bypass/no-redirect/200/identity fetch rules,
   no-service-worker deploy rule with persistent-client residual folded
   into the accepted split-view residual. Ceremony checklist rewritten to
   execute build -> pin -> deploy -> fetch-verify; served-manifest signing
   question RESOLVED (non-authoritative, no signing).
8. **D5:** versioned additive `intake` provenance block on
   ProvenanceEnvelope (cn-model minor) carrying consent affirmation +
   asserted time + linkage digests - the consent linkage now survives the
   purge in a schema that can represent it; sweep-manifest survival
   contract (manifest SHA-256 + count anchored in the repo-committed
   sweep DECISIONS entry; retained life-of-dataset; non-PII).

Round 4 (narrow, per the reviewer's own scope): durable owner, marker
recovery, atomic relay admission, ledger observation intervals, versioned
post-sweep provenance. ADR-005 remains DRAFT until it passes.

## D-064 (2026-07-24) - ADR-005 round 4: FAIL judged valid; fourth amendment - decision-inbox protocol

Round 4 (narrow; review at
`_reviews/community-connector/2026-07-24_adr-005-remote-intake-round4.md`,
target HEAD 277c9e2) returned FAIL: one blocker (the round-3 decision-file
inbox had no idempotent admission - a crash could replay an approval into
a FRESH plan with new op ids; stale wizard decisions could contradict
completed ones), plus: non-prefix mixed log patterns and pending+marker
states unruled; the provenance "minor bump" contradicted cn-model's
accepts_schema (same-minor required at major zero) and the actual carrier
path (envelopes ride op payloads' modeled values, not fold-time
stamping); cutoff epoch unanchored to a confirmed config revision; one
"bounded KB" overshoot claim survived the sweep; three stale approval
contracts in the blueprint. All judged valid. The ledger mechanics and
bundle measurement were declared substantially closed.

Fourth amendment choices:

1. **Decision-inbox admission protocol (D4):** decisions are versioned
   messages (body-carried decision_id, payload binding,
   expected_review_state CAS premise, typed decision incl. clear_failed).
   Native apply admits via one table: deterministic order, binding
   check, decision_id dedup against history (every history entry records
   its decision_id), CAS staleness (typed stale_decision, retired
   unapplied), legal transitions only. Admission + plan + approved_intent
   are ONE atomic sidecar write, so a replay can never mint a second
   plan. Retire-after-durable into decisions/consumed/ tombstones.
2. **Contiguous-prefix rule (D4):** marker completion only when present
   ops are exactly a plan-order prefix and absent exactly its suffix;
   any hole/out-of-order -> terminal failed. Two new crash-table rows for
   pending+marker (with/without decision files); marker create-if-absent
   idempotent.
3. **Provenance scoping corrected (D5):** optional serde-defaulted field
   + global model PATCH bump (a minor bump would reject 0.1.x data);
   single-workspace atomic deployment stated as the assumption; the
   block is constructed by plan_approval inside the modeled values -
   fold-time-stamp claim withdrawn. Sweep manifest now stored
   REDUNDANTLY (two controlled locations, restore-checked at the sweep).
4. **Cutoff epoch (D6/D3):** anchored to the platform's successful
   acknowledgement of the exact revision + documented propagation bound
   (or stated assumption) + the enforced Workers request limit; D3's
   "from that moment" corrected.
5. **Overshoot honesty (D6):** "bounded KB" removed; overshoot is
   unquantified within the consistency window; quota + billing ceiling
   numeric values are a deploy gate.
6. **Blueprint sweep:** submit_ops design-intent line, four-state enum,
   and facade-level plan assertions replaced with the native-owner
   equivalents; decision.rs now carries the admission table.
7. **Served manifest NOT deployed** (smallest safe rule; ceremony
   aligned).

Round 5 (narrow, per the reviewer): decision-inbox crash/idempotency,
prefix validation, provenance-version migration, manifest recovery,
cutoff receipt semantics, corrected blueprint. ADR-005 remains DRAFT.

## D-065 (2026-07-24) - ADR-005 round 5: FAIL judged valid; fifth amendment - revision CAS, digest projection

Round 5 (narrow; review at
`_reviews/community-connector/2026-07-24_adr-005-remote-intake-round5.md`,
target HEAD e19f7bf) confirmed closed: prefix rule (in the ADR), hard-cap
sweep, provenance patch-bump direction and modeled-value carrier (checked
against accepts_schema/op.rs/fold.rs), redundant sweep manifest. Still
FAIL on: (1) state-only CAS has an ABA hole through legal
failed->clear_failed->pending and the history schema contradicted the
dedup rule; set_aside_note had no defined state effect; stale decisions
had no durable disposition; (2) in-payload batch_digest was circular;
(3) D3 claimed "provable/enforced" beyond the cutoff's assumption branch;
the ceremony destroyed old-key recovery copies before the drain in the
leakage path and used pre-amendment timing in the planned path; (4) the
blueprint had not carried atomic admission or prefix-only recovery
through. All judged valid.

Fifth amendment: monotonic `sidecar_revision` on every rewrite +
`expected_sidecar_revision` in every decision (revision+state CAS closes
the ABA); ONE authoritative history-entry schema (decision_id, canonical
message digest, type, prior/resulting state+revision, outcome
admitted|stale|illegal|replay; same id + different digest = loud
conflict); every decision type's exact effect defined (set_aside_note
keeps pending, bumps revision); durable stale/illegal history entries
BEFORE retirement + tombstone/history startup reconciliation;
`batch_digest` = SHA-256 over the pre-link projection (planned ops with
intake.batch_digest omitted), per-op durable-log digests after
population, consumers named; two-branch cutoff honesty (documented bound
= enforced; observed-bound assumption = stated accepted residual or
defer destruction) + revision-currency recheck at epoch completion +
one destruction rule for planned AND emergency rotations (ceremony 8.2
no longer destroys recovery copies early); blueprint carry-through
(atomic admission+history+plan+intent, retire-after-durable tombstones,
prefix-only + negative tests, pending+marker rows, file-conflict
narrowing); ceremony open-question manifest sentence fixed. Round 6
(narrow) pending; ADR-005 remains DRAFT.

## D-066 (2026-07-24) - ADR-005 round 6: findings 2-5 closed; two defects fixed; sixth amendment

Round 6 (narrow; review at
`_reviews/community-connector/2026-07-24_adr-005-remote-intake-round6.md`,
target HEAD e7728a6) CLOSED round-5 findings 2-5 (pre-link batch digest,
cutoff honesty + ceremony destruction unification, blueprint
carry-through, manifest sentence). Two valid defects remained in the
fifth amendment itself: (1) BLOCKER - replay/stale/illegal audit writes
advanced the same revision the decision CAS reads, so a crash retry (the
reviewer supplied the exact sequence) could invalidate and retire a
decision that was current when authored - non-idempotent; (2) MAJOR -
the single history schema could not encode the approval transaction's
own required events (preflight failure back to pending, completion,
digest conflict).

Sixth amendment: (a) two counters - physical `sidecar_revision` (every
rewrite, bookkeeping only) vs semantic `decision_generation` (the CAS
authority; advances on admitted decisions incl. note-only/clear_failed
and on state-changing transaction events; NEVER on stale/illegal audit
entries); (b) same-digest replays retire against their already-durable
original decision event with NO new write; (c) history gains
`event_kind`: decision events (outcome admitted|stale|illegal; dedup
target) vs transaction events (intent_completed | preflight_failed |
durable_conflict, linked by admitting decision_id, never dedup targets);
(d) both reviewer-supplied failure sequences are mandatory pure
admission tests in the blueprint. Round 7 is scoped by the reviewer to
ONLY these two defects, with the four closures not to be reopened.
ADR-005 remains DRAFT.

## D-067 (2026-07-24) - ADR-005 round 7: both round-6 defects closed; seventh (surgical) amendment

Round 7 (two-defect scope; review at
`_reviews/community-connector/2026-07-24_adr-005-remote-intake-round7.md`,
target HEAD 7892227) walked both mandated sequences CLEAN: the crash
replay is transparent (writeless same-digest retirement; generation
stable across audit writes) and stale-before-current admits correctly;
the writeless-replay rule creates no new hole (stale/illegal originals
are valid dedup targets). Remaining: one MAJOR - recovery's
hole/out-of-order `approved_intent -> failed` transition had no
transaction event in the tagged union - and one MINOR stale
`revision+state CAS` phrase in the blueprint. Seventh amendment: fourth
transaction variant `durable_inconsistency` (generation-advancing,
serialization-tested, carried through blueprint approval.rs/seam
tests/recovery), and the blueprint wording fix. Implementation gates
noted by the reviewer (digest-bound tombstone reconciliation,
fault-testing all decision outcomes) are recorded in the blueprint's
test lists. Round 8 is verification-only on these two edits.

## D-068 (2026-07-24) - ADR-005 ACCEPTED after eight adversarial rounds

Round 8 (verification-only; review at
`_reviews/community-connector/2026-07-24_adr-005-remote-intake-round8.md`,
target HEAD 729251f) returned PASS-WITH-NOTES: both round-7 edits closed,
one MINOR text note (a stale three-outcome parenthetical), applied in
this commit, with the reviewer's explicit recommendation to then mark
ADR-005 ACCEPTED without reopening the round-4..7 closures. Status
flipped to ACCEPTED.

Process record: the D-056.1 "one adversarial round" became EIGHT
(rounds 1-7 FAIL-and-amend, round 8 pass). Every round's load-bearing
findings were verified against the actual files/code before judgment and
none was rejected; the design hardened materially each round (browser
trust model; native durable owner; idempotent decision inbox with
semantic generation CAS; two-kind history; receipt ledger; enforceable
rotation cutoff; canonical bundle measurement; consent-carrying
provenance that survives the purge sweep). The full review trail (8
files) lives in the out-of-repo review lane. Standing implementation
gates named by the reviewer (canonical digest golden vectors, quiescent
provenance deployment, fault-injection at every protocol boundary,
digest-bound tombstone reconciliation, provider-bound evidence in the
deploy runbook, ceremony rehearsal) are carried in the intake-pipeline
blueprint and MUST be green in check-all before the D-059.8 deploy bar
can be met. Acceptance UNLOCKS: the P3.5/P3.6 intake-pipeline
implementation (blueprint at docs/blueprints/intake-pipeline.md) and,
separately gated, the relay implementation (mandate item 3). The deploy
bar itself is unchanged.

## D-069 (2026-07-24) - Intake approval ops: attributes ride inside EntityCreate

Blueprint step 4 implementation (52e274b) deviates deliberately from the
blueprint's earlier op-list sketch ("EntityCreate unowned + AttributeSets
+ EdgeCreates"): cn-store's fold validates REQUIRED attributes at entity
creation, so a bare EntityCreate followed by AttributeSet ops would
quarantine at create. plan_approval therefore builds ONE EntityCreate op
carrying the fully populated entity (attributes as AttributeInstances,
each carrying the D5 intake provenance block alongside the entity's own).
Multi-op batches still arise from multi-entity/edge submissions later;
the seam's prefix/recovery semantics are unaffected. Recorded in the
module header and the blueprint (aligned in this true-up commit). Edge
creation from the pilot form's org linkage stays deferred to the wizard's
merge-by-hand flow.

Implementation position at session close: blueprint steps 1-4 of 11
landed (78e9aae seam, 661b02c provenance block, 73e228b queue formats +
admission + recovery, 52e274b near-dup + plan_approval); check-all 7/7
green at every commit; next is step 5, the `cn intake apply` CLI.

## D-070 (2026-07-24) - Step 5 `cn intake apply`: implementation choices under blueprint authority

Blueprint step 5 landed (1db3cd0): the CLI is the native durable owner
per ADR-005 D4 with recovery, admission, transaction, and I12 report.
Nontrivial choices made inside the blueprint's discretion, recorded:

1. **The persisted plan carries the ops verbatim.** `ApprovalPlanRef`
   gains `ops: Vec<Value>` (the planned `Operation` values). Recovery
   under a durable intent must re-append the absent suffix WITHOUT
   regenerating ids (ADR-005 D4); op ids and digests alone cannot
   reconstruct the ops (entity ids and envelopes live inside them). The
   sidecar's "complete plan" is now literally complete.
2. **Run-level halting.** The lost-decision-state and binding-mismatch
   recovery rows, a failed sidecar rewrite, and a corrupt persisted plan
   HALT the whole run before any admission (stop-the-line; nothing is
   guessed). Quarantines, stale/illegal audits, id-reuse conflicts, and
   tombstone anomalies are recorded loudly, processing continues, and
   any of them forces a failure exit code so attention is unmissable.
3. **Queue layout fixed** as the contract the step-8 FSA adapter and the
   later relay puller write into: `<id>.record.json`,
   `<id>.sidecar.json`, `<id>.reviewed`,
   `decisions/<id>.<uuid>.json`, `decisions/consumed/`, `corrupt/`,
   `intake.lock`, `*.tmp-<uuid>` temps.
4. **The OS file lock IS the liveness check** (fs4): a held lock proves
   a live process, the OS releases it at process death, so a stale lock
   FILE never wedges the queue and takeover without liveness proof is
   impossible - satisfying the ADR's stale-lock rule by construction.
5. **Kind resolution:** a payload-carried `kind` field wins, else the
   `--kind` default (person, the pilot form). A wrong kind surfaces as
   preflight_failed via template validation - never a silent write.
6. **No template argument:** apply reads group and template from the
   folded ops log (the source of truth, I2), eliminating the
   wrong-template-file failure mode.

Deps fs4 + tempfile added after a RustSec/OSV/KEV audit (clean; fs4 1.x
API drift caught by the audit before first compile). Six integration
tests cover the decide -> apply -> reload round trip, worktree refusal,
writeless crash replay, stale serialization, approved_intent recovery,
and the authority-matrix preflight denial. check-all green.

## D-071 (2026-07-24) - Step 6 facade: dedup gap filled, five-arm verdict, shared kind rule

Blueprint step 6 landed (b47ab49): the read-only cn-api/cn-wasm intake
facade (BOUNDARY_VERSION 0.2.0) with the no-leak extension test proven
against a REAL projection (trust-granted governance sees the hidden
graph candidate; facilitator and anonymous never do; queue-side matches
still surface). Choices recorded:

1. **The blueprint's `dedup.rs` had never landed** (steps 3-4 shipped
   without it); it exists now. The verdict enum has FIVE arms, not the
   sketch's four: `transport_conflict` (same receipt id, different
   ciphertext hash) is distinct from semantic `conflict` because ADR-005
   D4 gives it different handling (retain relay copy, alert - never a
   no-op). Blank client-controlled submission ids never match each other.
2. **Near-dup facade takes a request DTO** carrying the template-driven
   `name_attrs`/`affiliation_attrs` (templates have no "name role"
   concept for the core to infer); the graph side is NEVER
   caller-supplied - it is the viewer's projection computed in-core.
3. **`validate_record` reuses the plan path** (throwaway ids), so the
   review UI's report is definitionally identical to the apply-time
   report (I2). Integrity failures (checksum, unknown major) are typed
   error envelopes, not validation findings.
4. **Kind resolution and the pilot default ("person") are single-sourced
   in cn-ingest** (`resolve_kind`, `DEFAULT_PILOT_KIND`); the CLI and
   facade both consume them (was CLI-local per D-070.5).

## D-072 (2026-07-24) - Human rulings: consent boilerplate authorized as draft; checkpoint = convention; ad hoc demos as authorization pathway

The human ruled on the two standing queue items (Q-CHK/Q-TEXT):

1. **Consent text:** the D-023 solo correctness pass will happen later.
   In the meantime the session is authorized to CREATE boilerplate
   consent/intake text matching the software's actual functionality and
   intentions (derived from the 2026-07-24 consent draft package). All
   such text stays PLACEHOLDER/DRAFT-marked and the CLAUDE.md
   community-facing-text gate is unchanged: nothing is shown to a
   community member before human review; the D-023 sign-off remains the
   bar for build/synthetic -> real use.
2. **Committee timing:** the collective checkpoint's moment is the
   convention itself (2026-09-14). Additionally, individual ad hoc
   meetings where the human demonstrates the software IN ADVANCE can
   lead to further authorization pathways. Consequences: August internal
   pilots stay CONDITIONAL (no recorded checkpoint before convention
   unless an ad hoc pathway produces one); demo-readiness of the wizard
   + synthetic fixtures becomes a sequencing priority (it directly
   enables those meetings). Engineering stays on synthetic data
   throughout - unchanged.

## D-073 (2026-07-24) - Step 8: core-built queue bytes; the app never computes a digest

The queue formats carry embedded checksums that must byte-match the
core's recomputation. Reimplementing serde-canonical serialization in
TypeScript would duplicate format authority and invite silent
divergence, so cn-api/cn-wasm gain two PURE BUILDER exports -
`intake_stage_record` (checksummed record + initial pending sidecar,
core-generated UUIDv7 record_id) and `intake_build_decision`
(core-generated decision_id and format version; the app supplies only
the CAS premise it observed and the decision content). These are
compute, not mutation: nothing durable exists on the in-memory
boundary, so the blueprint's no-approval-write rule stands untouched;
the app remains a create-only writer of core-composed bytes (I2). A
facade test proves the built decision ADMITS against the built sidecar
end to end. Checksum verification is order-independent (recomputed from
the parsed struct), so the app may re-serialize the returned objects
without breaking verification.

The FSA adapter (app/src/state/intake.ts) implements the create-only
contract (refuse-if-exists, write, read-back byte compare), the
best-effort in-browser guard (FSA grants no ancestor access, so only
the granted directory is probed - the native CLI guard stays
authoritative), the wizard refusal rule (no decisions against
approved_intent or unreadable sidecars), and the read-only dashboard
scan. Intake state is a new store slice that survives group reloads
(the queue belongs to the facilitator's ops directory, not the group
session).

## D-074 (2026-07-24) - Step 9 wizard: reject carries its reason; viewer_roles affordance

Blueprint step 9 landed (the P3.5 wizard). Two format/surface decisions:

1. **`DecisionType::Reject { reason }` (required).** The blueprint
   mandates a rejection reason and the record persists under D-059.11,
   so the WHY must persist too - but the ADR-005 D4 decision format's
   reject was a bare variant with nowhere to put it. The variant now
   carries `reason` (message body), and the admission table maps it into
   the history entry's `reason` field exactly as set_aside_note's note.
   Additive within the 0.x queue format; the wizard requires a non-empty
   reason before staging a reject.
2. **`viewer_roles` boundary export.** The wizard must show itself only
   to facilitator-or-governance viewers (an affordance; the core
   enforces regardless), but the app cannot recover roles from the
   hashed viewer fingerprint. cn-perm gains `viewer_role_names` (the
   viewer's OWN roles - their own authorization context, no third-party
   disclosure) and cn-api/cn-wasm expose `viewer_roles`; main.ts mounts
   the wizard only when the roles include facilitator or governance.

The wizard implements the blueprint section-5 surface: ops-directory
grant with the best-effort guard, queue dashboard (state counts, oldest
pending age, staged-decision count with the `cn intake apply` + reload
instruction), the P3.6 entry form staging through the create-only
adapter, and the review view running the three read-only core checks
(validation, dedup, near-duplicates) with approve / reject-with-reason /
set-aside-note / clear-failed decisions as create-only decision files.
Entry and approval remain two distinct recorded acts (D-030). FSA handle
persistence across sessions (IndexedDB) is deferred to the step-11
polish pass - the facilitator re-grants the folder per session for now.

## D-075 (2026-07-24) - Step 10 pii-scan tripwires: scoping and self-test

The intake tripwires (ADR-005 D4, blueprint section 7) landed: queue-shaped
path rules (*.record.json, *.sidecar.json, *.reviewed - flagged anywhere,
any content), the "queue_record_version" JSON data marker, and the
"secret-encrypted" key-envelope marker. Scoping choice recorded: .rs/.ts
sources and .md docs are exempt from the CONTENT markers only (they name
and describe the format keys; a real staged queue file never arrives as
source or markdown) - path rules apply to every file. The scanner file
itself is content-exempt by name (it carries the regexes). A new
`pii-selftest` check-all member (12 members now) GENERATES six positive
marker-bearing fixtures in the session temp dir at runtime, asserts each
trips its rule plus one exemption negative, and removes them - the
tripwire is proven live on every full run without anything marker-shaped
ever entering the repo. Tripwires remain defense in depth under the I1
process boundary, not enforcement.

## D-076 (2026-07-25) - Implementation adversarial round 1: FAIL, amended

The mandatory round on the steps-1-11 diff (reviewer gpt-5.6-sol; review
at _reviews/community-connector/2026-07-25_intake-pipeline-impl-round1.md,
target HEAD 8bcce9a) returned FAIL: four blockers, eight majors. Every
load-bearing finding was verified against the files before judgment;
none was rejected. Amendments landed in three commits (d1b5f47 core/CLI,
9630d1b app, plus the scripts/docs commit carrying this entry):

- F1 (BLOCKER, confirmed against ADR-005 D4 "re-run step 2"): recovery
  mode is now selected by durable presence - ALL-ABSENT plans re-run the
  full first-attempt step (preflight + validation gate); only partially
  appended batches complete under the intent marker; a recovery fold
  quarantine is a loud halt. Crash-after-denial integration test added.
- F2 (BLOCKER): decision reviewer must parse as PersonId and equal the
  authorized --facilitator; mismatches refused (reviewer_mismatch).
- F3 (BLOCKER): no swallowed IO - directory-flush failures recorded;
  Windows rename semantics documented honestly; app reads are tagged and
  scan issues surface on the dashboard.
- F4 (BLOCKER): review selection lives in the store; the FSA handle is
  owned by the state module's effects layer; the wizard holds no
  operational state.
- F5: stop-the-line halting (pre-pass on halt-class rows incl. the new
  marker-with-no-files -> HaltLostDecisionState; immediate stop on a
  failed approval recovery). F6: verify_persisted_plan cross-field
  checks incl. pre-link batch-digest recomputation. F7: sorted-key
  canonical digests (Value-tree serialization) in cn-ingest AND the
  seam, with pinned golden vectors. F8: digest-bound tombstone
  reconciliation + tamper test. F9: typed submission-schema validator
  (allowlist, versions, consent shape, caps, control characters) gating
  every first-attempt approve; empty reject reason ILLEGAL in core.
  F10: queue format 0.2.0 (reject wire-shape change; no deployed 0.1.0
  writer existed); nested extras on ApprovalPlanRef/HistoryEntry. F11:
  lifecycle-parameterized wizard mount + IndexedDB handle persistence
  landed. F12: the self-test now writes fixtures to disk and runs the
  REAL scan loop; the blanket .md content exemption is replaced by an
  explicit five-file doc allowlist (a renamed queue file as .md now
  trips); the exemption negative uses a quoted marker.

Recorded divergences (not silently claimed): the Windows write primitive
remains std::fs::rename (MoveFileEx REPLACE_EXISTING without
WRITE_THROUGH) with the ADR's own durability qualification documented at
the primitive; the staged-mode reader path of pii-scan is exercised by
the pre-commit hook itself rather than the self-test; the app still has
only demo + snapshot load paths, so the wizard's lifecycle binding is
proven on the demo path. D-068 gate positions after this round:
canonical-digest golden vectors GREEN; digest-bound tombstone
reconciliation GREEN; fault-injection EXPANDED (denial-crash, tamper,
validation arms) but not yet exhaustive per boundary; quiescent
provenance deployment, provider-bound runbook evidence, and the ceremony
rehearsal remain OWED - the D-059.8 deploy bar is unchanged and unmet.
Round 2 (verification) follows this commit.

## D-077 (2026-07-25) - Implementation adversarial round 2: FAIL, amended

Round 2 (verification; review at _reviews/community-connector/
2026-07-25_intake-pipeline-impl-round2.md, target HEAD 12c91f8) closed
F2/F4/F5/F6/F7/F8 and returned FAIL on the remainder. Every load-bearing
finding verified before judgment; none rejected. Amendments:

1. **Durable intent ordering (was the recorded Windows divergence,
   ruled BLOCKER-grade - accepted).** The queue now uses the ADR-005 D4
   primitive literally: MoveFileExW with REPLACE_EXISTING |
   WRITE_THROUGH on Windows (windows-sys), rename + parent-directory
   fsync FAILING CLOSED on Unix - for every queue rename (sidecar
   writes, decision retirement, quarantine moves). The intent marker can
   no longer be outlived by a later op-log fsync; the degraded-platform
   WARN is gone because the primitive replaced it.
2. **Sticky recovery quarantine.** A planned op that is durable AND
   quarantined by the startup replay - or quarantined by a recovery
   fold - now lands as the terminal `durable_inconsistency -> failed`
   transaction event (durable disposition), never a transient halt a
   later all-present run could reinterpret as success. Two-run
   integration test added.
3. **Fail-closed IO everywhere flagged:** recover_approval propagates
   validate_record errors; the app distinguishes NotFound from IO/
   permission failures (createOnly can no longer proceed past an
   ambiguous existence check), decisions-directory enumeration failures
   are reported, IndexedDB persist/restore failures are visible and
   typed ("granted"/"none"/"failed"); the pii-scan readers return
   typed results and an unreadable file is a READ FAIL violation
   (self-test case added).
4. **Full value contract (F9).** attr_value_from_json maps EVERY
   template type (date, geo point/region, link with format, media);
   validate_submission type-checks each field against its AttrDef (enum
   membership, tags cap 20 + homogeneity, shape validity via the same
   mapping approval uses) so nothing the planner would drop escapes a
   rejecting finding. The pilot's contact_email link field is proven to
   survive approval. The form's advisory caps now mirror the core's
   (2000 bytes text, 20 tags). The review panel renders BOTH validation
   reports.
5. **F10 tail:** extras preservation proven on plan and transaction
   carriers too. **F5 note:** tombstone reconciliation skipped after a
   pre-pass halt (no false anomalies).

Still-open by design (recorded, not silently claimed): the app's only
interactive load path remains the dev demo - blueprint step 9's
production mount is NARROWED to "bound to every interactive load path
that exists," with the August-pilot build owing the production path
(the absent path fails closed); IndexedDB paths have no node-env tests.
D-068 gates unchanged from D-076 except fault-injection coverage grew
(two-run quarantine, denial-crash, READ FAIL). Deploy bar unchanged and
unmet. Round 3 verification follows.

## D-078 (2026-07-25) - Implementation adversarial round 3: FAIL, amended; step-9 contract amended

Round 3 (review at _reviews/community-connector/
2026-07-25_intake-pipeline-impl-round3.md, target HEAD ca4e714) closed
F1 (recovery reauthorization + sticky quarantine now match ADR-005 D4 -
the reviewer found no remaining crash window) and confirmed the durable
rename primitive, and returned FAIL on the residue. All verified;
amendments:

1. **Native fail-closed predicates (F3 blocker):** the worktree guard,
   directory enumeration, and quarantine moves now use fallible metadata
   operations - only NotFound means absent; any other IO error refuses
   loudly (I3). Record/sidecar/decision reads distinguish IO failure
   (run refuses - it cannot classify what it cannot read) from parse/
   checksum corruption (the quarantine row), and the corruption reason
   reaches the I12 report. The app's .git probe refuses unverifiable
   roots the same way.
2. **Durable notices (F3/F11):** persistence warnings ride a new
   `notices` state field that scans never clear (only a directory
   change does); a reducer-level FINAL-STATE test proves the warning
   survives the scan that follows it. The dashboard renders notices.
3. **Geo contract end-to-end (F9):** the form now encodes the core's
   canonical raw geo shape ("lat, lon" -> {lat, lon}; anything else ->
   {name}); check_field validates geo objects deeply (exactly {lat,lon}
   or {name}; region names get the full hazard checks; unknown keys
   rejected). NFC Unicode normalization (unicode-normalization crate)
   is applied to stored text/tags/region names per the blueprint rule.
   The app cap is measured in UTF-8 BYTES matching the core
   (TEXT_MAX_BYTES, multi-byte regression test). form_version and
   consent_text_digest get hazard checks; non-sha256 digest shape is a
   WARNING (synthetic fixtures carry marker digests by design).
4. **F12 honesty note:** the self-test cleans up BEFORE claiming
   removal and fails if cleanup leaves fixtures behind.
5. **Step-9 contract AMENDED in the blueprint itself** (the reviewer's
   requirement): the mount is bound to every interactive load path that
   exists; the production/pilot path is owed with the August pilot
   build, where the same mount and the synthetic decide -> apply ->
   reload rehearsal are mandatory before pilot use. Fails closed today.

Still open and honestly recorded: browser-level IndexedDB tests, the
production interactive path itself, .rs/.ts/archive content bypasses in
the tripwire (defense-in-depth disclaimer stands), and the D-068 deploy
gates (fault-injection breadth, quiescent provenance deployment,
provider runbook, ceremony rehearsal). Round 4 verification follows.

## D-079 (2026-07-25) - Implementation adversarial round 4: FAIL on one residue, amended

Round 4 (review at _reviews/community-connector/
2026-07-25_intake-pipeline-impl-round4.md, target HEAD 426bb3c) closed
F9 outright (full end-to-end value contract incl. geo, NFC, byte caps),
confirmed persistence visibility, the self-test ordering, and judged
the step-9 blueprint amendment coherent - and failed on exactly one
remaining item: `ensure_marker` still used the error-collapsing
`exists()` and would have accepted a directory or symlink squatting on
the `.reviewed` name as the durable marker, silently losing the
lost-decision attention state on a later crash. Amended: the marker
check is a fallible `symlink_metadata` match - a regular file counts,
NotFound creates atomically, and any other object or IO error halts
loudly (I3); a CLI integration test proves a directory squatter halts
the run with nothing admitted, retired, or appended. Also added per the
round's recommendations: focused core regressions (geo exact-key and
range rejection, hostile region names, NFC-normalized storage, the
warning-only synthetic digest shape) and app advisory byte checks for
tag members and geo place names (core stays authoritative). The
reviewer's explicit judgment: with this predicate fixed, the remaining
open set is exactly the four honestly recorded debts (browser
IndexedDB/FSA tests, the August-pilot production path per the amended
step 9, tripwire source/archive bypasses under the defense-in-depth
disclaimer, and the D-068/D-059.8 deploy gates) and PASS-WITH-NOTES
becomes warranted. Round 5 verifies.

## D-080 (2026-07-25) - Intake implementation ACCEPTED: round 5 PASS-WITH-NOTES

Round 5 (review at _reviews/community-connector/
2026-07-25_intake-pipeline-impl-round5.md, target HEAD fbfced3) returned
PASS-WITH-NOTES: the round-4 marker residue is closed (symlink_metadata,
regular-file-only, loud on everything else; the reviewer found NO
remaining error-collapsing predicate on the intake authority path), the
value-contract regressions and app advisory checks are sound, and the
open set is exactly the four recorded debts. The blueprint steps 1-11
implementation, as amended by D-078, is ACCEPTED. The full 12-member
verification battery was executed at fbfced3 by the verification owner
(the reviewer was read-only), all green.

Process record: the mandatory round became FIVE (rounds 1-4
FAIL-and-amend at D-076/D-077/D-078/D-079, round 5 pass). Every round's
load-bearing findings were verified against files before judgment and
none was rejected. The design hardened materially: recovery
re-authorization for all-absent plans, reviewer-identity binding, the
literal WRITE_THROUGH rename primitive, sticky durable-quarantine
disposition, fail-closed native and app predicates end to end,
sorted-key canonical digests with golden vectors, digest-bound
tombstones, the typed submission-schema validator with the complete
per-type value contract (incl. geo and NFC normalization), durable
persistence notices, and the honest tripwire self-test. The review
trail (5 files) lives in the out-of-repo lane.

STANDING DEBTS (visible, unclaimed): (1) browser IndexedDB/FSA
behavior tests; (2) the production interactive load path + mandatory
synthetic decide -> apply -> reload rehearsal (amended step 9, owed
with the August pilot build, BEFORE pilot use); (3) tripwire
source/archive content bypasses under the defense-in-depth disclaimer;
(4) the D-068/D-059.8 deploy gates (exhaustive fault injection,
quiescent provenance deployment, provider-bound runbook evidence, human
ceremony rehearsal) - the deploy bar remains unmet and unclaimed.

## D-081 (2026-08-11) - Relay sealed-box binding: RustCrypto `crypto_box` (Phase A step 1)

Trigger: relay blueprint (docs/blueprints/intake-relay.md) section 1 requires a
Rust sealed-box implementation that opens what libsodium.js `crypto_box_seal`
produces in the browser (ADR-005 D3). The blueprint fixed the selection
CRITERIA, not the winner, and left the choice to the cross-implementation test
vectors: (1) sealed-box interop with libsodium.js proven by test vectors;
(2) pure Rust (no C build dependency); (3) maintained/audited/widely used;
(4) minimal API surface.

Options: (a) RustCrypto `crypto_box` with its `seal` feature; (b) `dryoc`
(pure-Rust libsodium port); (c) `libsodium-sys` (C bindings). (c) fails
criterion (2) outright (adds a C toolchain dependency to the workspace and to
the pilot PC's trusted computing base, the exact surface the sealed-envelope
design minimizes) and was rejected. Between (a) and (b): docs research
confirmed `crypto_box`'s `seal` feature IS libsodium's sealed box - the feature
pulls in `blake2` precisely for the `blake2b(ephemeral_pk || recipient_pk)`
nonce derivation `crypto_box_seal` uses. With interop achievable either way,
`crypto_box` wins the remaining criteria: RustCrypto is more widely used and
audited than dryoc's single-maintainer port (3), and `crypto_box` is the more
minimal surface - keygen, seal, unseal, nothing else (4) - versus dryoc's full
libsodium port.

Choice: `crypto_box` 0.9 (`seal` feature). Companions: `blake2` (BLAKE2b-256
fingerprint), `zeroize` (SecretKey hygiene), `argon2` (Argon2id passphrase KDF
for the secret-key file), `crypto_secretbox` (XSalsa20-Poly1305 secretbox for
that file - the ceremony's specified cipher; Rust-only, no interop needed since
the browser never reads secret-key files), and `base64` (key-file
salt/nonce/ciphertext encoding; the base64 dep was deliberately deferred from
the step-3 envelope work to this binding). The crypto module lives in
cn-ingest (pure crypto + byte-level envelope (de)serialization; no file or
network I/O - ADR-005 D1 module fence). Interop is not asserted by faith: the
committed fixture fixtures/crypto/sealed-box-vectors.json is produced by
libsodium.js (scripts/generate-crypto-vectors.js, a manual dev tool, NOT in
check-all) and tests/crypto_vectors.rs (which IS in check-all) proves every
libsodium-sealed box opens in Rust and the fingerprints match.

Implementation note: `crypto_box`/`crypto_secretbox` pull getrandom 0.2 for
OsRng. cn-ingest compiles into the wasm bundle (cn-wasm -> cn-api -> cn-ingest),
so the Cargo.toml gates `getrandom = { features = ["js"] }` to
`cfg(target_arch = "wasm32")` - without it the wasm build fails to compile
(the RNG code is present even though the wasm facade never calls it).

Strongest surviving objection: pinning `crypto_box` 0.9 pins the RustCrypto
0.5-era `aead`/`rand_core` 0.6 stack, which now trails the workspace's
rand_core 0.9 / getrandom 0.3-0.4 lines, so the tree carries multiple
getrandom/rand_core majors. This is cosmetic (cargo coexists semver-major
versions; the wasm js feature is gated correctly) and reversible: the binding
is a thin module behind our own PublicKey/SecretKey/Keypair types, so a future
swap to a rand_core-0.9 `crypto_box` (or dryoc) touches one file and re-runs the
same vectors. The interop guarantee - the thing that is expensive to get wrong -
is the property the fixture locks down regardless of which crate provides it.

## D-082 (2026-08-11) - Puller receipt-classification boundary rules (Phase B step 4)

Trigger: implementing the D6 reconciliation classifier (`cn-ingest`
`reconcile::classify_receipt`, blueprint intake-relay section 6.1 post-loop)
required two calls the ADR-005 D6 precedence text leaves unstated - the exact
TTL+margin boundary and the meaning of a missing age. Both decide whether a
receipt is flagged EXPIRED (a claim that drives a broadcast re-solicit) versus
INTEGRITY-ALERT (an ambiguous state re-checked next run), so neither can be left
to an accidental inequality.

Call 1 - boundary direction. ADR-005 D6 says blob-absent "before" TTL+margin is
an alert and "after" is expired; it is silent at the exact instant age ==
threshold. Options: (a) `age >= threshold` -> expired (boundary is expired);
(b) `age > threshold` -> expired (boundary is alert). Chose (a). The threshold
IS blob-TTL + consistency-margin, and per D6 the margin exists precisely so that
"expiry classification only becomes eligible after blob TTL + consistency
margin, which by construction leaves at least one full pull cadence" - the
margin is the conservatism buffer. At exactly the threshold that buffer is fully
elapsed, so the expiry claim is already evidence-backed; `>=` also stops a
receipt sitting exactly on the boundary from lingering in perpetual alert. This
matches the step-4 test contract ("age at/over TTL+margin -> expired").

Call 2 - missing/unknown age with the blob absent. Options: default to expired,
or default to integrity-alert. Chose integrity-alert. Expiry is a positive claim
that the observation window has fully elapsed; without an age we cannot assert
that, and D6 frames the alert as exactly the "ambiguity is stated and
investigated, re-checked once next run before alarming" state - the honest bucket
for an observation we cannot yet trust. Defaulting the other way would
manufacture an unevidenced expiry (and an unwarranted re-solicit).

Strongest surviving objection: call 1's strict-`>` alternative is equally
defensible under a literal reading of "eligible AFTER" the margin, and the two
differ only at a measure-zero instant. Accepted: the difference is one boundary
tick, both keep the margin as the real safety buffer, and the reserved-for-
early-ambiguity role of the alert state is preserved either way. The classifier
is pure and stateless, so revisiting the tick later is a one-line change plus a
test flip.

## D-083 (2026-08-11) - Pages intake form: outer-envelope base64 variant + build tooling (Phase C step 6)

Trigger: implementing the Pages intake form (`form/`, blueprint intake-relay
section 4; ADR-005 D2/D3/D8) surfaced two calls the ADR/blueprint leave open,
one of which a LATER, different session depends on.

Call 1 - outer-envelope base64 variant (a cross-session dependency). `envelope.rs`
deliberately keeps `OuterEnvelope.ciphertext` an opaque, undecoded string
("decoded only by the puller's crypto binding"); that decode is step 7's job, in
a future session, and nothing committed yet pins which base64 variant the outer
envelope uses (standard vs URL-safe, padded vs unpadded). Options: (a) standard
padded, RFC 4648 s4 (libsodium `sodium.base64_variants.ORIGINAL`); (b) URL-safe;
(c) unpadded forms. Chose (a). Rationale: `crypto.rs` already uses
`base64::engine::general_purpose::STANDARD` for its OTHER base64 fields (the
key-file salt/nonce/ciphertext), so standard-padded gives the whole crypto
surface ONE variant a future Rust reader expects, rather than two. The form
encodes with `sodium.to_base64(bytes, ORIGINAL)`; step 7's Rust decode must use
the STANDARD engine to match. Recorded here so step 7 does not have to
reverse-engineer it from minified JS.

Call 2 - build tooling: Vite multi-file, NOT vite-plugin-singlefile. Blueprint
4.3 leans minimal but explicitly permits Vite; the prompt recommended Vite +
singlefile. Deviation from singlefile: the blueprint CSP is
`script-src 'self'; style-src 'self'`, which cleanly admits EXTERNAL same-origin
assets but NOT inline scripts (inline needs a hash or 'unsafe-inline', a weaker
CSP). A fully-inlined single file would therefore force either an inline-script
hash workflow or 'unsafe-inline'; multi-file (index.html + assets/*.js +
assets/*.css) keeps the blueprint's exact CSP verbatim and matches D8's
multi-file manifest model (D8 lists "every deployable file"). Vite's
content-hashed asset filenames are deterministic per source commit + lockfile,
verified empirically by the double-build byte-identical check in
`scripts/build-form.ps1 -CheckReproducible`. libsodium's WASM is embedded as
base64 inside the JS chunk (the standard `libsodium-wrappers` build), so the
bundle stays dependency-closed with no runtime fetch (D3/D8) despite being
multi-file. `libsodium-wrappers@0.7.15` is the form's ONLY runtime dependency.

Strongest surviving objection: Call 1 leaves interop UNVERIFIED until step 7 -
this step ships an encoder with no Rust decoder to round-trip against, so a
variant mismatch would only surface later. Accepted: the choice is recorded
explicitly (not left implicit), the encoder is exercised by the sealed-box
wrapper tests (base64 round-trip; a committed standard-base64 fixture ciphertext
decodes to the right length under ORIGINAL, cross-checking the variant against
the libsodium.js generator's `Buffer.toString('base64')`), and full
cross-language interop is blueprint step 9's e2e job by design. For Call 2, a
single-file artifact is marginally simpler to reason about as "one pinned blob,"
but the manifest already pins the whole multi-file set with equal strength, so
nothing is lost.

## D-084 (2026-08-11) - Relay Worker test toolchain: cloudflareTest plugin + manual KV wipe (Phase C step 5)

Trigger: blueprint 5.4 and the step-5 prompt specified the Workers Vitest
integration as "Vitest + miniflare" and named `defineWorkersConfig` from
`@cloudflare/vitest-pool-workers/config` plus an `isolatedStorage` pool option.
The current package that actually installs - `@cloudflare/vitest-pool-workers`
0.21.0, the vitest-4 line whose peer requirement (`vitest ^4.1.0`) matches the
app's already-pinned `vitest 4.1.10` - removed BOTH: there is no `/config`
subpath export, no `defineWorkersConfig`, and `isolatedStorage` is no longer a
config key. The step-5 prompt explicitly directed resolving concrete tool calls
against "what's actually current and actually installs," not training-data
memory, so this is a resolve-it decision, not a blocker.

Options: (a) pin an OLDER pool version (the 0.8.x era) that still exposes
`defineWorkersConfig` + `isolatedStorage`, matching the prompt's literal API
names; (b) adopt the current v4 API - the `cloudflareTest()` Vite plugin fed to
`defineConfig` from `vitest/config` - and replace `isolatedStorage` with an
explicit per-test KV wipe.

Choice: (b). Pinning an old pool would drag vitest back to the v3 line,
conflicting with the app's pinned `vitest 4.1.10` (the workspace standard) and
with the current pool's own peer range; matching the live workspace toolchain
outranks matching now-stale API spellings in the brief. The `cloudflareTest()`
plugin takes EXACTLY the object that used to live at `poolOptions.workers`, so
the config's substance is unchanged: `wrangler: { configPath }` loads `[vars]`
and the KV binding from wrangler.toml, `miniflare.bindings` supplies the two
secrets (`CREDENTIAL_HASH`, `ADMISSION_ALLOWLIST`) with local-only test values,
and one KV namespace backs both prefixes. The dropped `isolatedStorage` is
replaced by a `beforeEach` in `test/setup.ts` that lists and deletes every KV
key (all prefixes) for a deterministic clean slate. Tests use the documented
unit-test dispatch (`import worker from "../src/index"`; call
`worker.fetch(request, env, ctx)` with `env` from `cloudflare:test`), which also
lets the write-order and forced-failure tests wrap `env.INTAKE_BLOBS` in a Proxy
to observe/inject KV behavior.

Adopted AS SPECIFIED (pre-reasoned in the step-5 brief, no separate
deliberation, recorded here only for traceability): the `GET /receipts`
union-of-both-prefixes response shape (the single listing route that surfaces
both the pulled/expired and the orphan-blob crash cases, D6); one KV namespace
with `blob:`/`ledger:` prefixes; the KV-backed fixed-window per-IP rate limiter;
and DELETE idempotency tracking the blob's prior existence (200 deleted-now vs
204 already-gone).

Strongest surviving objection: the manual `beforeEach` wipe is harness
scaffolding the removed `isolatedStorage` did natively, so a future pool upgrade
that restores or renames the option would leave dead cleanup code; and a wipe
that itself failed mid-loop could leave residue masking a real bug. Accepted:
the wipe is a few order-independent lines (it deletes all keys unconditionally),
and its correctness is transitively proven - the isolation-sensitive tests
(empty-listing, approximate-cap, per-IP rate limit) would fail loudly if state
leaked between cases. The pinned versions (wrangler ^4.120.1, pool ^0.21.0,
`@cloudflare/workers-types` ^5.x, typescript ^6.0.3) live in relay/package.json
and enter DEPENDENCIES.md at the step-11 docs true-up, not incrementally here.

## D-085 (2026-08-11) - `cn intake pull` CLI + D8 bundle verification (Phase D steps 7-8)

Trigger: implementing the native puller (`core/cli/src/intake/pull.rs`) and the
D8 bundle-verification module (`bundle.rs`, blueprint intake-relay section 6 and
8; ADR-005 D1/D3/D4/D6/D8) surfaced five calls the ADR/blueprint leave open.

Call 1 - `manifest_path` config field (blueprint 6.3 extension). The D8 manifest
is written OUTSIDE the deploy root and is NOT served by Pages (D8 no-deploy
rule), so the puller cannot fetch its CONTENT (the file list) from the origin;
it must read the local ceremony-pinned manifest file. Blueprint 6.3's config
spec pins only `manifest_hash`, not a path to the file. Options: (a) an implicit
path convention (e.g. `<key_dir>/dist.manifest.json`); (b) add an explicit
`manifest_path`. Chose (b): a clean, minimal field the ceremony operator sets
alongside the keys and config, avoiding a hidden convention. `manifest_pin.
manifest_hash` still guards that file against substitution (verify_bundle hashes
the file's exact bytes and compares BEFORE parsing it), so the added path widens
no trust surface.

Call 2 - in-memory (single-run) delete journal. The blueprint names a "local
delete-journal entry" for the D6 `DeletedByMe` reconciliation class without
fixing a persisted format. Chose an in-memory `HashSet<String>` of receipt ids
deleted THIS run. Reconciliation runs in the same process immediately after the
main loop, so `DeletedByMe` is correct within a run. Across runs: a blob this
run deleted whose ledger also expired simply will not appear next run; one whose
ledger persists classifies `Expired` (age past TTL+margin) - both correct
end-states. A persisted cross-run journal buys only a transient, self-correcting
distinction (`DeletedByMe` vs `Expired`) for one interval, at the cost of a new
durable format and its own corruption modes. Deferred; recorded so a future need
(e.g. suppressing re-solicit for a just-deleted receipt) reopens it deliberately.

Call 3 - `CONSISTENCY_MARGIN_SECS = 300`. Reconciliation's expiry threshold is
`blob_ttl_seconds + margin`; the D6 classifier treats a blob-absent receipt at
or past that threshold as cleanly `Expired`, below it as an `IntegrityAlert`.
KV is eventually consistent, so a receipt that JUST crossed its TTL may briefly
still be listed, or a fresh ledger may briefly precede its blob. Five minutes is
generous for KV convergence at pilot scale and matches the ledger-TTL margin the
Worker already builds in (step 5). A too-small margin cries false alerts at the
TTL boundary; a too-large one delays a real "blob vanished early" alarm - 300s
sits comfortably in between for a pilot.

Call 4 - `--queue <path>` as a CLI arg, not a config field. Blueprint 6.1 leaves
the queue-root source open ("add `--queue`, or add `queue_root` to PullConfig").
Chose `--queue`, matching `cn intake apply`'s exact interface: the puller and the
applier operate on the SAME queue root under the SAME single-instance lock, so
one flag spelling for both is the operator-muscle-memory win. The config still
holds every OTHER puller path (key_dir, credential_path, manifest_path); only the
queue root - the one path shared with a sibling subcommand - is a flag.

Call 5 - testable core split + a widened CLI test surface. `execute_pull` is a
pure-ish core (queue lock, bundle verify, main loop, reconciliation) taking an
injected `&dyn RelayHttp` and a `fetch` closure; `run` does only the out-of-band
loading (config, credential, passphrase-decrypted key) and wires the real ureq
clients. To let `core/cli/tests/intake_{bundle,pull}.rs` inject closures/mocks at
the Rust level (the whole point of the `fetch`/`RelayHttp` seams - a subprocess
cannot receive a closure), `intake` is now `pub mod` and `bundle`/`pull` expose
their test-facing items as `pub`. The CLI crate is `publish = false`, so widening
its API costs no semver surface; the alternative (spawning the binary against a
throwaway local HTTP server) could not exercise the transport-error -> Degraded
path or the "relay never contacted on a bundle halt" assertion.

Strongest surviving objection: Call 1's `manifest_path` and Call 4's `--queue`
both mean the ceremony/ops surface now spans a config file AND a CLI flag AND a
key directory - three places an operator can misconfigure. Accepted: the deploy
runbook (step 10) enumerates all three in one checklist, and each fails LOUD and
early (unreadable manifest -> `BundleResult::Failed` halt; unsafe/missing queue
root -> `refuse_unsafe_root` refusal; key-pin mismatch -> halt before any network
call), so a misconfiguration never silently degrades into wrong behavior. For
Call 2, the in-memory journal means a `DeletedByMe` signal does not survive a
crash mid-run, but a crashed run stages nothing it did not also verify and
deletes nothing it did not also stage, so the next run re-derives the same
classification from durable facts (the staged records and the relay listing).

## D-086 (2026-08-11) - Puller claim-verifier review dispositions (relay steps 7-8)

Trigger: an independent claim-verifier pass over the steps 7-8 landing (13b8879)
confirmed every headline claim by re-running the battery (fmt/clippy/test/pii
green, counts exact, module fence clean, Cargo.lock additive, D-085 accurate) and
surfaced issues the worker's self-review missed. Six were fixed as atomic commits
(F1 precondition order, F2 D6 orphan-blob flagging, F11 timestamp-parse
diagnostic, F6 configurable envelope cap, F10 HTTP timeouts, F4/F5 conflict +
reconciliation + CLI-entrypoint tests). Three dispositions need a durable record;
the rest are queued for the mandatory adversarial round.

Call 1 - F6 config-shape change (`max_envelope_bytes`). The puller previously
capped the outer envelope at the fixed `DEFAULT_MAX_ENVELOPE_BYTES` (8 KiB),
independent of the relay's deploy-configured `MAX_BLOB_SIZE_BYTES`. A puller cap
BELOW the relay's silently rejects a valid, consented submission the relay
accepted - lost consented data over a config mismatch (privacy). Options: (a)
leave it fixed and document "never raise the relay cap"; (b) add a config knob.
Chose (b): a new `max_envelope_bytes` field on `PullConfig`, serde-defaulting to
8 KiB so a config written before the field existed still parses and behaves
identically (I7 unknown-minor tolerance). This extends blueprint 6.3's config
shape (as `manifest_path` did in D-085). BINDING deploy-runbook constraint (step
10): puller `max_envelope_bytes` >= relay `MAX_BLOB_SIZE_BYTES`. The relation to
the puller's own 8 MiB HTTP read cap (a raise above it would be silently
truncated by the read) is F7, deferred to the round.

Call 2 - F8 semantic-conflict blob retention (was only in commit text). On a
semantic Conflict (same submission id, different bytes) the puller stages BOTH
copies and does NOT delete the relay blob - identical to the transport-conflict
rule, which the blueprint states explicitly. The blueprint spells out retention
only for the transport case; extending it to the semantic case is the
conservative choice: a conflict is an anomaly demanding facilitator attention
(ADR-005 D4 "never a drop"), and retaining the relay-side ciphertext preserves
the evidence for out-of-band re-pull/inspection rather than deleting it the
instant both copies are staged. Cost: the blob lingers to its TTL. Accepted for
the pilot; the round may revisit whether a staged-both conflict should delete.

Call 3 - F3 rotation old-key catch-up: recorded limitation, NOT built. ADR-005 D3
rotation has two halves. The enforceable cutoff (the relay admission allowlist
rejects new submissions to a retired key) is the binding D3 outcome and it is
implemented (step 5). The puller's old-key CATCH-UP (decrypting
already-submitted-but-not-yet-pulled envelopes for one TTL after a cutover) is
NOT: `PullConfig` is single-key (one `key_dir`, one `key_fingerprint_pin`,
exact-match rejection), and post-cutover an old-key run hard-fails bundle verify
(the manifest pins the new key's fingerprint). Options: (a) build a multi-key
puller now (multi-key config, try-each-key decrypt, per-key bundle verification,
rotation-window semantics); (b) record the limitation and defer. Chose (b): the
pilot uses a single key with no rotation before the convention, so multi-key
support is speculative generality (YAGNI) touching ADR-005 D3. Recorded here and
as an adversarial-round item; ANY real rotation requires the multi-key puller
design FIRST. This does not weaken the binding cutoff, which stands.

Strongest surviving objection: Call 1 adds yet another operator-tunable to a
surface (D-085) already spanning a config file, CLI flags, and a key directory,
and a mis-set `max_envelope_bytes` (e.g. below the relay cap, or above the read
cap) reintroduces the exact silent-drop the knob exists to prevent. Accepted: the
default is the safe 8 KiB, the field's doc-comment and the deploy runbook both
pin the >= relation, and the failure mode is a loud per-receipt "outer envelope
parse failed" error (never a silent stage-skip). For Call 3, deferring means the
system cannot rotate keys until the multi-key design lands - but rotation is out
of pilot scope by construction, and the limitation is now explicit rather than a
latent surprise mid-rotation.

## D-087 (2026-08-11) - Archived handoffs exempt from pii-scan marker-content rule

Trigger: the 2026-08-11 true-up archived the live `HANDOFF.md` to
`docs/archive/handoffs/2026-08-11-relay-7-8-dispatched.md`. That handoff's DONE
history quotes the D-075 tripwire NAMES ("queue_record_version",
"secret-encrypted") when describing the intake work. `HANDOFF.md` is on the
pii-scan marker-content-exempt list (D-075: the docs that record the tripwires
are allowed to name them); the archive directory was not, so pii-scan raised a
KEY MATTER violation on a byte-for-byte historized copy of an exempt doc.

Options: (a) a per-file allowlist entry each true-up (the email allowlist does
not cover marker violations anyway, and this recurs every archive); (b) redact
the marker names from each archived copy (mutilates the faithful history); (c)
exempt `docs/archive/handoffs/*.md` from the CONTENT-marker rule only. Chose (c):
an archived handoff IS a historized `HANDOFF.md`, the same document class already
exempt, so it records the same tripwire names for the same reason. This is not
the forbidden "exempt all markdown" relaxation - it is one bounded directory of
historized handoffs. The path, email, and queue-PATH rules still apply to every
file in it, so real key material or staged queue data could never hide in an
archived handoff (the exemption skips only the "does this text NAME the markers"
check, which is exactly what a handoff legitimately does).

Strongest surviving objection: a content exemption is a security-boundary
loosening, however narrow, and the pii-scan doctrine prefers allowlist entries
over rule changes. Accepted: the allowlist is email-only and structurally cannot
clear a marker violation, so the exempt-file list is the only mechanism for this
false-positive class (it already holds six such docs); the addition is a glob
over one directory whose contents are, by construction, copies of the already-
exempt live handoff. `pii-scan -SelfTest` still passes (rules trip, the
exemption negative holds). D-075's tripwire intent is unchanged for every
non-handoff file.

## D-088 (2026-08-11) - Durable owner reconciles the two intake timestamp conventions (relay step 9)

Trigger: blueprint step 9's form-to-graph e2e (`scripts/e2e-remote-intake.ps1`,
built by the post-true-up 2026-08-11 session and left uncommitted; picked up and
run this session) drove a SYNTHETIC remote submission all the way through and
FAILED at the final `cn intake apply` with a preflight `validation_failed`:
"consent_affirmed_at missing or not an integer". A real defect in the shipped
remote pipeline (steps 1-8), caught exactly where step 9 is meant to catch it -
`form/src/envelope.ts` had explicitly deferred "full cross-language interop
verification" to this step.

Causal chain: the two intake paths emit the same consent/capture timestamps in
DIFFERENT JSON types. The in-app form (`app/src/ui/forms/model.ts`) emits epoch-ms
NUMBERS; the remote form (`form/src/envelope.ts`) emits ISO-8601 STRINGS because
the Rust `InnerPayload` fields are typed `String` (a number is a hard deserialize
failure at the puller). The puller stages the decrypted inner payload VERBATIM
(`stage_remote_record` = `serde_json::to_value(inner)`), so a remote record's
`consent_affirmed_at` is a JSON string. But the durable owner's `approval.rs`
required an integer in TWO places: `validate_submission` (`Value::as_i64` -> hard
error) and the `IntakeProvenance` builder (`as_i64().unwrap_or(0)`). Net: no
remote submission could ever be approved, and even past validation the consent-
affirmation instant that ADR-005 requires to survive the purge sweep would
silently store as `Timestamp(0)`. Each half had tests (in-app integer fixtures;
remote ISO envelope-PARSE tests) but the two were never integration-tested
through apply until now.

Options: (a) normalize ISO -> epoch-ms at staging (`stage_remote_record`), one
canonical stored shape; (b) teach the durable owner to accept BOTH conventions;
(c) change the remote wire format to emit integers.

Choice: (b). (c) is a non-starter - `InnerPayload`'s `String` type makes a numeric
wire value a hard deserialize failure, and the ISO wire form is the deliberate
D-083 design. (a) is worse than it looks: the puller computes the in-loop semantic
dedup `payload_hash` over the ISO inner payload (`canonical_digest(&inner)`) while
`QueueRecord::new` computes `record.payload_hash` over the STORED payload - normalize
the stored payload and those two digests diverge, so a repeat of the same submission
is misclassified as a SEMANTIC CONFLICT instead of a replay. (a) also mutates the
stored client assertion. (b) is localized to `approval.rs` - the single place the
two client conventions must reconcile into modeled values - touches no checksum,
hash, or dedup path, and keeps the staged record a faithful copy of what the client
asserted. Implementation: a `timestamp_epoch_ms` helper accepting `as_i64()` OR a
parseable ISO string, used at all three sites; the hand-rolled ISO parser was
lifted from the CLI's `pull.rs` into `cn-model` (`parse_iso8601_utc_to_unix_ms`,
shared by the puller AND the durable owner) rather than duplicated. Regression:
a cn-ingest unit test stages a remote-shaped ISO record and asserts it validates
and lands a real (non-zero) epoch in `IntakeProvenance`; the opt-in e2e now passes
seal -> POST -> pull(real HTTP) -> approve -> apply -> export end to end.

Strongest surviving objection: the durable store now holds two representations for
one semantic field (integer for in-app, ISO string for remote), a latent smell - a
future consumer that reads a raw payload timestamp with a bare `as_i64` gets `None`
on remote records and could silently mishandle it. Accepted: the durable owner is
the sole authority that turns raw payloads into modeled values, and it now coerces
both; the coercion is documented on `timestamp_epoch_ms` as the required reader for
any new raw-payload timestamp. A uniform-wire cleanup (normalize at both form
boundaries so the stored payload is single-typed) is deferred, not foreclosed.
Note: this fix reconciles the timestamp TYPE only; the app-side facilitator review
view rendering a remote record's timestamps is display-only, outside the e2e assert
path, and is flagged for the step 9-11 adversarial round.

## D-089 (2026-08-11) - Relay steps 1-11 adversarial round: ACCEPT-WITH-FIXES

Trigger: all 11 relay blueprint steps had landed (D-081..D-088 + steps 9-11), and
the blueprint mandates an adversarial round on the whole implementation diff before
acceptance (the relay is permission-adjacent). Ran as a FIVE-reviewer read-only
panel, each with a refute-it mandate over its slice plus the carried backlog:
R1 crypto+keygen, R2 puller+bundle+reconcile, R3 relay Worker, R4 form, R5
durable-owner+D-088+app+e2e. Every material finding was re-verified on disk by the
conductor before disposition (verify, don't trust the reviewer).

Verdict: ACCEPT-WITH-FIXES. No confidentiality or PII-leak blocker. Every
security-critical core HELD under refutation: the crypto trust root (JS<->Rust
sealed-box vectors 5/5, fingerprint parity across all three impls, per-file KDF
salt/nonce, versioned key file); the zero-trust relay (no plaintext read, no
oracle to an unauth caller, constant-time auth, correct write ordering, no
key-namespace escape, 43/43 green); the D1 module fence (EMPIRICALLY clean -
cargo tree across all nine cn-* crates shows no network/TLS crate); and the
permission model + durable owner + the Rust side of D-088. The D-088 app-review-
view residual that partly motivated the round is NOT a defect: the review view
formats no payload timestamp at all (robust by omission).

The ~22 real findings clustered in three themes; all clear ones were FIXED this
arc as four atomic commits (fbcab71 crypto/keygen, bd689c1 puller/parser, 0ab9e5f
relay, 5b12060 form) plus two conductor follow-ups (generator version source, a
keygen USAGE doc true-up):
- THEME A, two DEPLOY-BLOCKING form defects the node-only suite structurally could
  not catch (the form would not run in a browser): R4-1 the CSP `script-src 'self'`
  blocked the WebAssembly libsodium needs to seal -> added 'wasm-unsafe-eval';
  R4-2 no Vite base -> absolute /assets/ refs 404 on the Pages project subpath ->
  base='/community-connector/'.
- THEME B, I3 silent-failure discipline (the recurring one): R3-1/2/3 the relay
  swallowed its OWN malformed-credential / ledger-parse / list-truncation ->
  class-only loud logs + a ledger schema version (I7) + full cursor pagination;
  R1-1 a Fingerprint::from_str PANIC on multibyte input -> typed error; R1-4 a
  remove_file that swallowed its result and then lied it succeeded -> truthful;
  R2-3 a silently-dropped relay cursor -> loud halt; R5-1 the D-088 date parser
  accepted impossible calendar dates (Feb-31 -> a WRONG epoch) -> reject via a
  civil-date round-trip.
- THEME C, conflict-path under-design: R2-1 after a transport conflict, decrypted-
  PII records RE-STAGED UNBOUNDED on every pull (classify_dedup first-match-wins) ->
  prefer an exact (receipt, ciphertext_hash) replay before concluding a conflict.
- Plus hardening carried in the same commits: R1-2 passphrase zeroization, R1-3 a
  stderr-not-a-TTY guard on the printed secret-key backup, F7 a config cap
  assertion, R3-4 CORS on /submit error paths, R3-5 raw-ArrayBuffer verbatim
  storage, R4-4 the vectors version mislabel (+ the generator so a regen stays
  consistent), and three coverage tests (R4-3 the D-030 consent gate, R5-2
  envelope.ts ISO-string faithfulness, R3-6 the 404 auth/missing-id identity).

Two decisions made in the round:
1. R4-2 deploy target = the blueprint's documented GitHub Pages PROJECT subpath, so
   base='/community-connector/'. The human confirmed the form is NOT ready to be
   public, so this aligns the form to the plan-of-record WITHOUT committing to going
   live; base is re-confirmed at real deploy (a custom-domain root site would drop
   it). The deploy bar (D-059.8) stays unmet.
2. R2-2 (conflicting records are staged but not durably LINKED, so the facilitator
   wizard - a separate process reading the queue - could approve BOTH halves and
   double-admit one conflicted submission) DEFERRED as a design follow-up. It is an
   anomaly path (only after a transport/semantic conflict), the fix needs a durable
   conflict marker on the review sidecar plus wizard/apply handling, and R2-1
   already closes the unbounded-restaging half. Owed before real ingestion of any
   conflict-prone data.

Deferred / recorded LIMITATIONS (not fixed): R1-5 Argon2id m_cost is unclamped on
read (offline DoS on a hand-tampered key file); R1-6 an Argon2 doc-vs-impl params
divergence; R4-5 no build guard against shipping the localhost default relay
origin; F12 real-HTTP redirect/timeout/non-200 behavior is covered only by mocks +
one happy-path pull (test debt); R5-3 the review view does not display consent
evidence directly (I12 completeness); F3 the single-key puller cannot drain
pre-cutover envelopes post-rotation (D-086, no rotation in pilot scope). Plus the
two step-10 pre-deploy gaps: no GitHub Pages deploy workflow authored, and the base
target to be re-confirmed. All are on the human/deploy queue, none blocks pilot-
readiness on synthetic data.

Verified NOT-A-DEFECT (recorded so they are not re-raised): the D1 fence; F13
(Degraded-only-on-first-file matches ADR-005 D8 verbatim); R2-4 (the DeletedByMe
reconcile arm is unreachable in a single run because staging precedes the delete
journal); the D-088 app-review-view residual; CF-Connecting-IP (the CF edge sets it,
not client-spoofable, and the fallback fails safe); the constant-time hash compare;
the blob key-namespace prefix (no escape to ledger:/ratelimit:); and the 409
form_out_of_date allowlist "oracle" (recipient fingerprints are world-readable by
design, D3).

Strongest surviving objection: the panel was Claude subagents, not an independent
external adversary (Codex), and reviewers sharing the implementer's model may share
its blind spots. Pointedly, the two deploy-blocking form defects escaped EVERY prior
per-step review AND the node-only test suite because no one had run the built form
in a real browser - exactly the class of defect the automated gates cannot see.
Accepted with a named mitigation: the findings are concrete and file:line-verified
(the conductor re-confirmed the HIGH ones on disk - CSP string, absolute asset refs,
the dedup ordering, the parser rollover), the fixes are small and independently
tested (check-all 12/12, relay 47/47, form 47/47), and a REAL-BROWSER smoke of the
built form is now an explicit pre-deploy gate owed before the D-059.8 bar can clear.

## D-090 (2026-09-12) - group-template schema_version bump: PATCH (0.1.1), not MINOR (0.2.0)

Trigger: S-R2 (ATNI convention template) ruled "an additive `aliases` field with a
minor version bump" for `schemas/group-template.schema.json`. `cn_model::accepts_schema`
(`core/crates/cn-model/src/lib.rs:45-52`, proven by its own test at
`core/crates/cn-model/tests/blueprint.rs:374`, `assert!(!accepts_schema(&Version::new(0,
2, 0)))`) requires the SAME minor while major is 0 - a literal 0.1.0 -> 0.2.0 bump
would be silently rejected by every real reader (`cn-schema::validate_schema_version`,
`cn-api::wire.rs:91`), including the dev-app WASM path this session's own fixture had
to load through. S-R2's scope fence names only `schemas/`, `fixtures/`, `app/scripts/`,
and this session never touches `core/` to teach the model a new accepted minor.

Decision: bump the group-template schema's declared version to **0.1.1** (a PATCH
bump in the 0.1.x line), not 0.2.0. This mirrors the identical precedent already in
this repo: `MODEL_SCHEMA_VERSION` moved 0.1.0 -> 0.1.1 for the optional
`IntakeProvenance` block (`core/crates/cn-model/src/lib.rs:33-37`), also a PATCH bump
for an additive-optional field, for the exact same `accepts_schema` reason.
`schemas/group-template.schema.json`'s own `schema_version` property changed from a
hard `const: "0.1.0"` to an `accepted_schema_version` $ref/pattern (`^0\.1\.[0-9]+$`),
matching the sibling schemas (op-log, story-path). The new optional top-level
`aliases` array (shape only, D-093c/D-094c; matching logic is R3's, not this
session's) is added as a recognized property. `fixtures/templates/atni-convention.
template.json` declares `schema_version: "0.1.1"` and does NOT populate `aliases`
(the Rust `GroupTemplate` struct has `#[serde(deny_unknown_fields)]` and has no
`aliases` field yet, so an instance that populated it would fail to parse on the
real core - confirmed by smoke-loading the fixture through `cn-wasm`'s Node package,
which parsed and projected 87 entities / 291 edges without error). `research-network.
template.json` and `fisheries-committee.template.json` are untouched at 0.1.0
(still valid under the same 0.1.x line; no reason to touch unrelated fixtures).

This is a `CLAUDE.md` "yours to decide autonomously" item (schema drafts while
versions are 0.x). Strongest surviving objection: a literal reading of "minor version
bump" was not honored to the letter. Reversible if a future session teaches
`accepts_schema` a wider acceptance window and wants the true semver-minor instead.
Nothing ships on the un-browser-tested path while the bar stands.

## D-097 (2026-09-12) - Viz quick wins from the Codex rendering critique: tuning calls

Trigger: the human asked for the six "quick wins" in the gpt-5.6-sol review
`C:\dev\_reviews\community-connector\2026-09-12_threejs-viz-critique.md` (off-repo).
Landed as eight commits, 81af3f2..86f46fa. Each was screenshot-checked in headless
Chromium (SwiftShader WebGL renders correctly there, unlike the automated capture
S-R4b tried). Four calls went beyond the review's text:

1. **Layout radius 500 -> 340.** Swapping the inverted center/offset radii (the
   review's fix) put kind centers at 500, which pushed far clusters almost fully
   into fog and off the default frame. 340 keeps roughly the old overall extent.
   Every node moves; the median same-kind vs cross-kind distance test pins the
   intent. Known v0 limit: two kinds with nearby hashed centers still overlap.
2. **Fog density 0.0009 -> 0.00065.** Fit-on-load moves the camera from a fixed
   z=1100 to the true framing distance (~1300 on the research fixture), and the old
   density dimmed the whole framed graph. Far clusters still recede.
3. **Selected halo is a rim ring, not the resting glow at higher alpha.** The
   resting falloff is brightest behind the node; at `selectedAlpha` 0.8 it read as
   a solid disk merging with the node. The ring (detail-3 shell, 1.6x base halo
   scale) keeps the node legible inside it.
4. **Near-camera label size cap, depth-tested labels.** Not in the six. Clustering
   made focus flights land among close same-kind nodes whose world-sized labels
   filled the view, a regression the clustering itself caused, so it was fixed in
   the same pass. Screen-space collision (review C3) stays open.

Also found and fixed: presenter mode's graph region grew past the viewport (canvas
intrinsic-aspect fallback), a pre-existing S-R4b layout bug.

Open: no Codex review pass ran on these diffs (commits marked
`[unreviewed-by-codex]`); the S-R4b human visual gate is unchanged and now covers
these changes too. Strongest surviving objection: the tunings are judged on the
120-node research fixture only, not atni-convention or the 5,000-node scale.

## D-098 (2026-09-12) - Convention reveal on atni-convention; Codex diff review disposition

Trigger: the human asked (on the convention laptop) to make the reveal open the
atni-convention fixture via a `?group=` URL parameter, and to run the Codex review
of the D-097 commits. Commits 0cb27f0..290820d.

Reveal: `?group=` selects a synthetic dev fixture (research-network stays the
default); `scripts/reveal.ps1 -Group` defaults to atni-convention. The S-R4b "1
entity" finding was a viewer with no grant, not missing fixture data: the atni
fixture already grants governance to three synthetic people, and one of them sees
all 87 entities. No fixture change was needed. Running the real convention data
exposed four presenter problems, all fixed and screenshot-checked: the load fit
used a bounding sphere (graph at ~40% of the view; now a frustum fit of the
screen-plane extent); the beat caption covered the graph (fits now keep a 16%
bottom inset); kind beats only moved the camera (their kinds now take the existing
highlight role and the rest dims); committee rings seen edge-on read as pills
(torus nodes now face the camera, with sphere picking).

Codex review (`C:/dev/_reviews/community-connector/2026-09-12_viz-quickwins-diff-review.md`,
changes requested): both High findings verified and fixed with tests or browser
measurements (stale hover cache; drift wait rendering every frame), as were
Medium stale tooltip on rebuild, maxDistance clamping tall fits, halo refresh
upload cost, pointercancel, and the Low initial-fit snap (now a gentle fly;
snaps under reduced motion). Deferred, with reasons:

1. **Keyboard/Escape tooltip (Medium, brief deviation).** The tooltip stays a
   pointer-only, aria-hidden visual echo. The flat view and search already expose
   every name to keyboard and screen-reader users; a focusable graph-keyboard
   grammar is larger work than the convention window allows.
2. **index.ts over the I5 ~500-line guide (584 lines).** Hover moved out to
   `viz/hover.ts`; extracting the presenter controller and frame scheduler is a
   pure refactor with regression risk two days before the reveal. Revisit post-
   convention.
3. **Scheduler tests and a 5,000-node halo benchmark on Iris Xe.** Loop idling is
   measured in a real browser, not unit-tested; halo refresh cost is reduced but
   unmeasured at scale.

Human-only still: the S-R4b visual gate, now on the convention laptop's real GPU
and display with `pwsh scripts/reveal.ps1`. Commits since 81af3f2 that predate
this review carry `[unreviewed-by-codex]`; 290820d is the reviewed-and-fixed
point, and 0cb27f0..d0e15a0 postdate the reviewed range, so they remain
unreviewed.

## D-099 (2026-09-12) - Next-phase direction: convention end-to-end, planned on synthetic data

Trigger: the human's direction at the evening true-up. Paraphrased: a QR code takes
participants to a simple dark-mode ATNI-branded page (GitHub Pages if still the plan),
their entries are collected and assigned proper nodes, edges, and clusters, and they
enter the visual graph; the presenter gets simple keys (`c` committees, `o` orgs, `t`
Tribes, `s` state) and a few on-screen buttons; prepare the next phase for tests of the
system; add creative "wow" tasks.

Decision (planning only, nothing built): the next phase is
`docs/planning/NEXT-PHASE-convention-e2e.md`, sessions S-E1..S-E5 in
`SESSION_ROSTER.yaml`, plus the W1-W5 wow slice in the same doc. Every session builds and
rehearses on SYNTHETIC data. The direction does NOT resolve D-090c ("nothing goes live
for the convention"), does not clear any D-059.8 deploy-bar item, and does not open the
real-data gate; the plan carries both a synthetic-demo path and a go-live checklist, and
going live stays a human decision.

Findings the plan rests on (verified on disk by the director): an approved intake record
produces exactly one `EntityCreate` and no edges
(`core/crates/cn-ingest/src/approval.rs:602-624`, per the intake blueprint's "EdgeCreates
deferred"), and the ATNI template has no attribute through which a participant could name
a committee or organization - so "proper edges" is new, permission-adjacent work (S-E2,
mandatory adversarial round). `tribe` is free text and no `state` data exists, so `t`
and `s` need a human call: **D-099c (candidate) - what `s` means** (state of residence,
repurpose to the `single_tie` beat, or inert until decided). Brand colors and fonts come
from the human; nothing is invented or scraped.

Also recorded: the creative pass's privacy rule for the projector - a `nameOnStage` beat
flag, default off, so measure and constellation beats show counts rather than individual
names until an on-stage consent line exists (community-facing text, D-023 gate).

## D-100 (2026-09-12) - Presenter key `s` = state of residence (resolves D-099c); push authorized

Human ruling: `s` on the presenter keyboard means **state of residence**. The human
also authorized pushing local `main` to origin.

No state data exists today, so S-E3 adds it. Engineering defaults, reversible (schema
drafts at 0.x are ours to decide per CLAUDE.md):

1. An optional `state_of_residence` attribute on the atni-convention person kind:
   `enum` of US states plus DC, with "Outside the United States" and "Prefer not to
   say". Additive, with a schema PATCH bump per the D-090 precedent. The form and the
   wizard render it with existing widgets. It is a standard geographic list, not the
   community capability vocabulary that D-051 reserves to ATNI.
2. Default visibility `group`, never `public`; tier T1 like every pilot field (D-034).
3. On the projector the `s` beat is aggregate only: members grouped or counted by state,
   no names (the D-099 name gate), and states with fewer than 3 members shown as
   "fewer than 3" to avoid small-count re-identification. The threshold is a starting
   value for the human's visual review.
4. The synthetic fixture generator assigns synthetic states, weighted toward the ATNI
   region, so the beat has something to show. It stays synthetic data.

The state option list and the on-stage wording are community-facing text and go through
the D-023 review before any real use.

## D-101 (2026-09-12) - ATNI typography for Community Navigator surfaces

Human ruling (brand choice, Google Fonts):

| Role | Face |
|---|---|
| Titles | League Spartan SemiBold |
| Bold body text | League Spartan Bold |
| Subtitles | Lexend Medium |
| Body text, presentation style (large text: presenter mode, projector captions, rail) | League Spartan |
| Traditional body text (form fields, panels, reading text) | Calibri |

Engineering constraints and defaults, reversible:

1. **Self-host, never load from Google at runtime.** League Spartan and Lexend are SIL OFL
   1.1 and ship as bundled files (e.g. `@fontsource/league-spartan`,
   `@fontsource/lexend`, pinned exactly like the existing
   `@fontsource/atkinson-hyperlegible`). Three reasons: the intake form's CSP is
   `default-src 'self'` with no `font-src` exception (`form/index.html:23`); the offline
   snapshot must boot with zero requests (R8); and a request to a third-party font CDN
   would expose each participant's IP address to that CDN when they open the form.
   Adding the packages is a recorded dependency change (exact pins,
   `NOTICE-third-party` entries).
2. **Calibri cannot be bundled.** It is a Microsoft font, not on Google Fonts and not
   redistributable, so it can only be used when the viewer's machine has it installed.
   Stack: `Calibri, Carlito, <system sans>`. Carlito is OFL, metric-compatible with
   Calibri, on Google Fonts, and self-hosted like the others, so layout holds on phones
   without Calibri (most participants scanning the QR code).
3. **Subset for names:** Latin plus Latin Extended, as DESIGN_BRIEF item 22 already
   requires for the current face, so Indigenous and community names with diacritics
   render.
4. **The 3D graph labels are not silently switched.** They use Atkinson Hyperlegible
   (troika SDF) for legibility at distance. S-E1/S-E3 may propose League Spartan for
   presenter-mode labels, backed by a legibility screenshot at projector scale; the
   human decides.
5. The palette is still owed by the human; S-E1 keeps placeholder colors until then.

## D-102 (2026-09-12) - Traditional body text is Arial (supersedes D-101's Calibri row and item 2)

Human ruling, right after D-101: traditional body text (form fields, panels, reading
text) uses **Arial** instead of Calibri. The rest of D-101 stands.

Arial is also a proprietary Monotype/Microsoft font and cannot be bundled. It is
installed on nearly all Windows, macOS, and iOS devices, which covers far more
participants than Calibri did. Android generally does not ship it. Stack:
`Arial, Arimo, Helvetica, sans-serif`. Arimo is metric-compatible with Arial, Apache 2.0,
on Google Fonts, and self-hosted like the other faces (no runtime Google requests, per
D-101 item 1), so layout holds on devices without Arial. D-101's Calibri/Carlito stack is
superseded.

## D-103 (2026-09-14) - Convention sprint: A9 constellation choreography, pregenerated synthetic shape, and the picks that resolve source conflicts

Trigger: the human's 2026-09-14 directive to run the convention sprint as a long-running
autonomous workflow (discovery swarm, Sonnet sorting team, orchestrated execution) whose
output is the A9 finale of the General Assembly session on 2026-09-15: after a 30-minute
role-play focused on one Nation, the constellation shows that Nation as one node among
Relatives at ATNI, that shared goals connect them, and that the interconnection surfaces
the Tribes, allies, and affinities that make complex tasks achievable. Plan of record:
`docs/planning/CONVENTION-SPRINT-2026-09-14.md`.

Causal chain: a 22-agent discovery-and-sort workflow found the reveal mechanically ready
(fixture, presenter mode, reveal launcher) but no beat performs the A9 moment, the
fixture's person-to-person edges are arbitrary rather than priority-based, and the spine
still lists OQ-10 (live vs pregenerated) open with the deploy bar D-059.8 unmet. Three
skeptics then found: the synthesized caption "connected across shared priorities" was
an overclaim; a `focusEntityId` beat implies a camera flight that the stage's visual
language forbids ("outputs resolve in, no fly-ins"); the consent line is already recorded
in the spine as spoken, never a screen; measure captions at topN 8 would flood the screen;
and `beats.atni.json` has no validation, so a mistyped id fails silently.

Options: (1) the synthesizer's minimal path (three data-only beats, cue sheet,
verification; ~10 h); (2) that path plus the changes that make the story structurally
true (priority-derived edges in the synthetic fixture, opacity-only spotlight with a
camera hold, a shared-priorities beat, count-only measure captions, beat validation,
the S-E3 keys and rail, a bounded ATNI styling slice); (3) also attempt S-E2 edges at
approval on main before Tuesday.

Choice: (2), with S-E2 built only on a branch behind its mandatory adversarial round and
never merged without the human's recorded verdict. Recorded picks:

1. Tuesday's shape is pregenerated synthetic only, RECOMMENDED pending the human's
   OQ-10 answer; nothing live, nothing deployed, no real data.
2. `connected_to` person-to-person edges in the synthetic fixture derive from shared
   `areas_of_interest` tags (two or more), replacing the `(i*13+7)` formula, so a
   "shared priorities" claim on screen matches the data. Synthetic data only; the
   generator stays deterministic.
3. The one-node beat spotlights by opacity with the camera holding the full frame
   (new optional `camera` field on `PresentBeat`, default unchanged). This honors
   stage-flow.yaml's "no fly-ins" rule and is a stronger image than a flight: one lit
   node among many. The human may flip it to a flight by editing the beat.
4. The finale beat carries no filter and no measure so the whole constellation is lit
   through applause; the connectors (betweenness) beat is a floor beat, not an A9 beat.
5. Captions show counts, never names (D-099). Measure captions become "label - N
   highlighted"; explanations are not concatenated on stage.
6. The consent line stays spoken (spine decision "TSDF is spoken, never a screen");
   no caption carries it unless the human reverses that decision with a reason.
7. The cue sheet keeps the spine's single documented cue as the entry point and uses
   three Space presses across the last three sentences; a pre-switch at "These tools
   can be more" is offered to the human as an alternative, since it needs a spine edit.
8. The spotlight node's Tribe stays fictional until OQ-02 clears Makah; naming it is a
   one-line generator change.
9. Spine-side edits (OQ-10 resolution, tool-reference wording, QR sign-up lines) are
   drafted as a new reconciliation note beside the spine, never applied to its YAML.
10. Screenshots committed to this public repo capture the browser viewport only and are
    eyeballed by a human first; `pii-scan` never inspects pixels.

Strongest surviving objection: the presenter capability changes (camera hold, edge-kind
filter, count-only captions) touch the render path the human has not yet seen on a real
GPU, the night before the show. Mitigation: defaults preserve today's behavior, every
change ships with tests, the full loop and a browser walk run before the freeze, and the
human's real-GPU walkthrough remains the gate; any beat can be cut by deleting one array
entry.

### D-103 addendum 1 (2026-09-14, ~02:00) - Codex review dispositions and the stage name gate

Codex review (profile `review`, gpt-5.6-sol) of `6165d03..d3d2283` is at
`C:\dev\_reviews\community-connector\2026-09-14_convention-sprint-review.md`. One
blocking finding, seven advisories. Dispositions:

- **BLOCKING - person names rendered on stage via the label layer and hover** (beats
  that focus or emphasize people rendered synthetic `display_name` labels; hover showed
  full names). Accepted and FIXED in `0c2b756`: optional `labelKinds` on `PresentBeat`
  filters the lit set's labels to the named kinds in present mode; every authored beat
  now carries `labelKinds` without `person` and the beat-fixture test forbids `person`
  there; the hover tooltip is suppressed in present mode. Browser proof: zero person
  labels on every beat (48/48 assertions). Also folded in: the operator rail is hidden
  by default in present mode and `r` toggles it, so the projector shows results, not UI.
- **Advisory 1 - ATNI ground and text tokens apply to every mode, not only present
  mode.** Accepted as intended: the design system is the app-wide visual authority
  (D-101, digest); a mode-scoped ground would put two grounds in one product. Recorded,
  no change.
- **Advisory 2 - beats file has no `schema_version` and no runtime validation (I7).**
  Deferred: `beats.atni.json` is app-internal presenter data, not an exchange format;
  the beat-fixture test validates it at build time against the fixture and the type.
  A versioned envelope is post-convention work (one small unit).
- **Advisory 3 - edge-derivation verifier outside the battery.** Accepted and FIXED in
  `5ce91d8`: `npm run check:fixture` is chained from `validate:templates`, which the
  `app-templates` check-all member runs; member count stays 12.
- **Advisory 4 - hardcoded caption counts.** Accepted and FIXED in `5ce91d8`: every
  number in a beat label is asserted against the fixture (people with a priority tie,
  entities per kind).
- **Advisory 5 - zero-result measure caption.** Accepted and FIXED in `0c2b756`:
  "label - 0 highlighted" on an empty measure, bare label only while pending; tested.
- **Advisory 6 - hotkey tests stop at the mapping.** Partly accepted: the B2 browser
  proof drives the real keydown, rail clicks, and rail visibility against the running
  app; a unit test that drives `handlePresenterKeydown` through the store is
  post-convention.
- **Advisory 7 - `app/src/viz/index.ts` over the I5 size threshold (~680 lines).**
  Deferred with an exception recorded here: splitting the renderer coordinator the night
  before the show reopens the path the human has not yet seen on a real GPU. Split
  after the convention (presenter coordination into its own module).

Also this addendum: the live intake rehearsal (CS-10) proved the ATNI template form,
the wizard mount, and `cn intake selftest --dry`; the queue-folder grant is a native
picker the human performs. It found a real defect: staging before a folder grant
fails silently because the dashboard returns before rendering `lastError`. Fixed in
the `fix(intake)` commit that follows.
