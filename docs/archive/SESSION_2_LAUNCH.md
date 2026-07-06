# Community Navigator: Session 2 Launch Prompt

> Paste this document into a fresh Claude Code session (Fable model, or the
> strongest available Claude per the succession rule) started in
> `C:\dev\community-connector`. It launches the DECISION SESSION: a
> comprehensive interview with the human to scope the remaining work, then a
> re-optimized execution plan. Read this whole document, then follow
> Section 1 in order. Conventions: hyphens, never em dashes.

---

## 1. Session sequence

1. Read, in order: `CLAUDE.md` (contract - gates, invariants, routing),
   `HANDOFF.md` (live state - outranks everything including this file),
   `docs/PROJECT_PLAN.md` (current plan), `DECISIONS.md` (D-001..D-018+).
2. LAND IN-FLIGHT WORK FIRST: check `.codex/viz-fixes-result.md` (S3-A
   visual fixes may have finished after this prompt was written). If it
   exists and is uncommitted: judge it, verify (app/: npm run typecheck,
   npm test, npm run build:smoke; root: pwsh scripts/pii-scan.ps1), take
   the accepting headed screenshot (playwright-cli open --browser=chrome
   --headed, dev server from app/ via npm run dev in background), judge
   against HANDOFF's four visual findings, commit atomically. If a codex
   process is still running, let it finish while interviewing.
3. Deliver the 60-second brief (Section 2 below, updated against HANDOFF
   if state moved).
4. Conduct the interview (Section 3). Use AskUserQuestion for the
   structured choices (max 4 options per question; mark the recommended
   option "(Recommended)" first); free-form follow-ups in chat. Work
   through the parts IN ORDER - later parts depend on earlier answers. The
   human may skip anything; every question lists the default that then
   applies. Do not exceed two AskUserQuestion calls per part; prefer
   conversation for the open questions.
5. Execute the post-interview protocol (Section 4): record decisions,
   re-scope the plan, present Execution Plan v2, get one confirmation,
   then begin (or park if the human says park).
6. Standing mechanics throughout: atomic commits without asking; codex
   routing per docs/CODEX_GUIDE.md section 7 (grind = gpt-5.5 via -m
   override, review = gpt-5.5 high, --sandbox danger-full-access per
   D-008); re-arm the 8:00 AM safety cron; PII rules absolute.

## 2. The 60-second brief (verify against HANDOFF before delivering)

"In one long session, Community Navigator went from an empty folder to:
complete contract docs with a proven PII tripwire; three adversarially
reviewed ADRs; a full Rust/WASM core - permissions, event-sourced store,
graph queries - with a property test proving no data ever leaks above a
viewer's access; an evidence-based renderer decision measured on this
machine's GPU; and a working app that loads a synthetic community of 120
people and places and draws it as a 3D constellation (screenshots in
docs/design/screenshots/). ~57 commits, all verified. What remains: the
second half of the frontend, data ingestion, personal mode, and hardening -
about 10-13 sessions mapped in docs/PROJECT_PLAN.md. Today is the decision
session: your answers scope what v0.1.0 actually is."

## 3. The interview

### Part 1 - Mission and first use

**Q1.1 First real deployment.** Which community uses this first, and
roughly when? Options: (a) an ATNI committee pilot (the fisheries-style
template was built for exactly this); (b) CPF-RCN as a returning demo
(research-network template, migration recipe needed); (c) internal
sandbox only for now. RECOMMENDATION: (a) if a willing committee exists -
a real non-research community exercises R2 harder and earlier than
another research demo. Unblocks: Phase 4 ordering, Session E timing.

**Q1.2 The workflow that must shine in v0.1.** Options: (a) exploring the
network (viz-first, the predecessor's proven demo); (b) need-to-solution
routing ("who here can help with X" - the fisheries need_met_by path);
(c) self-managed profiles and sharing (personal mode). RECOMMENDATION:
(a) + (b) together - they demo without identity infrastructure and (b) is
this product's distinctive claim over the predecessor. (c) depends on
Part 3 answers. Unblocks: S3-B scope, Phase 5 priority.

**Q1.3 (open) What outcome makes the first demo a win?** Who is in the
room, and what should they say afterward? This calibrates polish targets.

### Part 2 - Deployment, distribution, and the backup risk

**Q2.1 Primary vehicle.** Offline single-file snapshot (predecessor's
proven mode) vs the live app. RECOMMENDATION: snapshot-first polish -
0.55MB currently, runs from a USB stick, no setup. Unblocks: S3-C
priority, Phase 6 targets.

**Q2.2 Where does the live app run?** Facilitator's laptop in meetings /
a shared kiosk / individuals' own devices. RECOMMENDATION: facilitator
laptop for v0.1 (matches Q3 recommendation and avoids device-support
scope). Unblocks: perf targets, personal-mode urgency.

**Q2.3 BACKUP (highest-leverage one-liner - risk, not feature).** The repo
exists ONLY on this laptop. Options: (a) private GitHub/GitLab remote;
(b) git bundle copies to a second drive/share on a schedule; (c) accept
the risk. RECOMMENDATION: (a) - one line from you satisfies the remotes
gate and removes the project's single worst risk. Unblocks: the gate;
nothing else.

### Part 3 - Identity and personal mode

**Q3.1 How does a person prove who they are** in a local-first app when
personal mode arrives? Options: (a) committee-issued claim codes
(governance hands each member a one-time code binding their record - no
vendors, works offline, sovereignty-aligned); (b) device-local
passkeys/OS keychain; (c) defer personal mode past v0.1.0 entirely.
RECOMMENDATION: (a), designed in Session D; it composes with (b) later
and commits to no external standard (that choice stays gated for the
network era). Unblocks: Session D, ADR-005/identity.

**Q3.2 Who is "governance" in the app on day one?** The facilitator? A
named committee officer? RECOMMENDATION: the group creator holds
governance until they grant it onward (matches the existing bootstrap
exception in cn-perm). Unblocks: wizard design (Session B).

**Q3.3 Does v0.1.0 NEED personal mode**, or is facilitator-managed data
with the viewer-switcher demo enough? RECOMMENDATION: keep Phase 5 but
time-boxed and AFTER a real committee has used facilitator mode - real
usage will reshape the sharing UX better than speculation. Unblocks:
whether Phase 5 gates v0.1.0 or becomes v0.2.

### Part 4 - Data governance and migration

**Q4.1 First real dataset + authority.** What data enters first, WHO
assigns its tiers (the Indigenous governance authority for that data),
and what does the consent/FPIC checkpoint look like in practice - a
motion in a meeting? A signed form? RECOMMENDATION: define the checkpoint
BEFORE any real ingestion is scheduled; fixtures remain the only data
until then. Unblocks: Session E scheduling, the real-data gate process.

**Q4.2 CPF-RCN migration timing.** Write-the-recipe-now (docs only) vs
wait until an ATNI pilot proves the tool. RECOMMENDATION: recipe written
during Phase 4 regardless (it costs one docs session and de-risks the
demo path); execution stays human-gated. Unblocks: Session E position.

**Q4.3 Tier language in the UI.** TSDF codes (T0-T3), ATNI's own
protocol vocabulary if it exists, or plain language ("kept by the
community" / "shared with trusted partners" / "public"). RECOMMENDATION:
plain language primary with tier codes secondary - but if ATNI has
existing data-governance wording, mirror it exactly. Unblocks: Session F,
detail panel copy.

**Q4.4 Provenance visibility.** How much of the IEEE-2890 chain should a
regular member see on a record - full custody chain, a one-line "added by
X from Y", or governance-only detail? RECOMMENDATION: one-line summary
for members, full chain for governance. Unblocks: detail panel design
(S3-B).

### Part 5 - UX, accessibility, and content

**Q5.1 Accessibility reality.** Known assistive-tech users in the target
communities? Elder-focused needs beyond WCAG (type size, simplified
mode)? Languages beyond English on the v0.x horizon? RECOMMENDATION
(absent specifics): WCAG 2.2 AA per the design brief, a font-scale token
wired from day one, single language. Unblocks: Session A scope.

**Q5.2 Group creation reality.** Facilitator-led wizard, or self-serve?
And is TEMPLATE authoring (new community types) in-app for v0.1 or a
JSON-file task? RECOMMENDATION: facilitator wizard; templates stay JSON
until two real communities have shipped. Unblocks: Session B scope.

**Q5.3 Stories.** Who authors them (facilitator? members?), and is
in-app authoring needed for v0.1 or is data-file authoring enough?
RECOMMENDATION: viewing in v0.1, authoring UI early v0.2 unless the demo
plan needs it. Unblocks: S3-C scope.

**Q5.4 Aesthetic check.** Show the human the latest screenshot from
docs/design/screenshots/. Does Hearthlight (warm night-sky constellation)
feel right for the communities involved? Any cultural considerations the
palette/shape language should respect or avoid? Unblocks: S3-A polish
direction.

### Part 6 - Ingestion

**Q6.1 What files do committees actually have?** Spreadsheets? Contact
exports? Meeting minutes? RECOMMENDATION: CSV-first ingestor with a
column-mapping step (Phase 4 already assumes this; confirm reality).

**Q6.2 Duplicate handling.** Auto-merge exact matches vs always queue
for human review. RECOMMENDATION: always queue; auto-merge nothing in
v0.1 (community data + provenance stakes beat convenience). Unblocks:
Session C / ADR-005.

### Part 7 - Process, budget, and cadence

**Q7.1 Autonomy level.** The bootstrap ran overnight autonomously,
including accepting REDESIGN verdicts and re-directing work. Options:
(a) same - full autonomy between decision sessions; (b) autonomous builds
but PARK anything a review calls REDESIGN until you look; (c) supervised
sessions only. RECOMMENDATION: (a) with (b)'s parking rule for
ARCHITECTURE (ADR-level) redesigns only - the record shows directed
judgment held up, but architecture pivots deserve your eyes.

**Q7.2 Spend.** ~20 Codex offloads and heavy Claude usage in session 1
(hit one limit reset). Comfortable, or should sessions be smaller/rarer?

**Q7.3 Cadence + failover.** Keep the 8:00 AM safety cron + usage
failover directives standing? Preferred session rhythm (overnight bursts
vs working hours)?

**Q7.4 License (standing one-liner).** PolyForm NONCOMMERCIAL 1.0.0 was
selected from your "polyform" answer - confirm, or name Internal Use /
Small Business instead. Default: Noncommercial stands.

### Part 8 - Retrospective (open discussion, not decisions)

- What in session 1's output (docs, aesthetic, plan, this interview)
  misses the mark?
- Anything about the predecessor demo's reception that should reshape
  this product's priorities?
- Anyone else who should see the plan or answer parts of this interview
  (committee members, ATNI staff)?

## 4. Post-interview protocol

1. Convert every answered decision to a DECISIONS.md entry (one each,
   dated, with the answer verbatim where short). Unanswered questions:
   note "default applies" - do NOT create entries for pure defaults.
2. Feed structural answers into their artifacts: Q3 -> Session D ADR
   inputs; Q4 -> Session E prerequisites + the real-data gate process
   written into CLAUDE.md's gate section if the human defined one; Q5/Q6
   -> Session A/B/C scopes in PROJECT_PLAN.
3. Rewrite docs/PROJECT_PLAN.md section 3 as "Execution Plan v2":
   re-scoped session list with the answers applied (cut/defer/reorder -
   especially whether Phase 5 gates v0.1.0 per Q3.3), each session with
   its acceptance criteria and Fable/Codex routing. Log the re-scope as a
   DECISIONS entry.
4. Present Execution Plan v2 to the human as a compact table + the three
   next sessions in detail. ONE confirmation question: "proceed on this
   plan?" Then either begin session S3-A/S3-B work immediately
   (autonomous per Q7.1's answer) or park cleanly per the session-end
   protocol.
5. Refresh docs/NEXT_SESSION.md (brief + remaining unanswered questions
   only) and retire this launch prompt: move it to docs/archive/ in the
   closing commit.

---

Begin with Section 1, step 1.
