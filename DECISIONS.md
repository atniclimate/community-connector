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

Predecessor commits auto-resolved to `patrick@atni57.onmicrosoft.com` (a machine
identity nobody chose - exactly the failure the brief flags). This repo sets
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
