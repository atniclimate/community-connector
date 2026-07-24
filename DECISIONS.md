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
