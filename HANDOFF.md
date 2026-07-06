# HANDOFF.md - Live State

> This file outranks session memory. Reading order for a new session: CLAUDE.md,
> then this file, then current-phase ADRs.

Last updated: 2026-07-06 afternoon, session 2 (the DECISION SESSION:
SESSION_2_LAUNCH interview Parts 1-5 answered, D-019..D-040 recorded,
Execution Plan v2 adopted, graph-networks deep research commissioned).

## Phase status

**Phase 0 - Bootstrap: COMPLETE.**
**Phase 1 - Domain model and ADRs: COMPLETE** (ADR-001..003 accepted).
**Phase 2 - Rust core: CLOSED** (six crates behind cn-api; permission
property test green; measurement gates recorded; closing review D-016).
**Phase 3 - Frontend: ~HALF.** Done: ADR-004 renderer (instanced layer),
design brief final, I4 state machine + wasm worker, theming pipeline,
base renderer landed + visual fixes DIRECTOR-ACCEPTED
(docs/design/screenshots/viz-fixes-2026-07-06.png).

## Session 2 outcomes (this session)

1. Decision interview conducted through Part 5; the human ended it there;
   Parts 6-8 defaults apply. Eighteen entries: **D-019..D-040**. The
   spine: ATNI Climate convention pilot (D-022 arc), snapshot-first
   (D-024), facilitator laptop (D-025), backup risk ACCEPTED (D-026),
   identity deferred (D-027), facilitator role in cn-perm (D-028),
   personal mode = v0.2 (D-029), form-respondents-only + QR joins
   (D-030), TSDF codes primary (D-032), demo-wide T1 with ATNI Climate
   as tier authority (D-034), accessibility deferred post-pilot (D-035),
   facilitator wizard (D-036), in-app story authoring in v0.1 (D-037),
   DESIGN sitting deferred (D-038).
2. **Execution Plan v2** written into docs/PROJECT_PLAN.md section 3
   (D-039): v0.1.0 = convention-pilot ready; 9-12 sessions; Phase 5 /
   Session D / Session A / Session F remainder -> v0.2.
3. Real-data gate process added to CLAUDE.md gates section (D-030/D-034).
4. NEXT_SESSION.md refreshed (brief + remaining questions Q-A..Q-F).
5. docs/SESSION_2_LAUNCH.md retired to docs/archive/.
6. Graph-networks deep research commissioned (D-040); report lands at
   **docs/research/graph-networks-report-2026-07-06.md**. Check it is
   present and committed; if missing, the research did not complete -
   re-run per D-040's scope.

## Exact next three actions

1. **Human sitting (preferred next):** read the research report alongside
   Execution Plan v2; shape plan v3; answer NEXT_SESSION Q-A (convention
   date) and Q-B (form platform); schedule the DESIGN sitting (D-038).
2. **Autonomous build (if no human):** S3-A2 "Constellation legible" -
   labels (troika, grind HIGH), motion/focus polish (grind medium),
   benchmark re-run vs p95 goal. Then S3-B "Explore + Route".
3. **FORM deliverable (docs, any time):** draft the intake form + ATNI
   template + consent email per D-023 for HUMAN REVIEW - it gates the
   consent email and needs no code. Do not send or publish anything.

## Open human gates

1. License variant: PolyForm Noncommercial 1.0.0 stands by default
   (D-019/Q7.4); one line changes it to Internal Use / Small Business.
2. (Standing) no remotes (risk explicitly accepted, D-026), no real data
   (pilot gate process now defined in CLAUDE.md), no spend without
   explicit instruction.
3. Community-facing text (form, consent email) requires human review
   before any use (D-023).

## Degraded modes / standing directives

- Codex runs use `--sandbox danger-full-access` (D-008 mitigations).
- Routing policy MANDATORY (docs/CODEX_GUIDE.md section 7); effort-match
  grind tasks (D-014).
- Atomic commits without asking (CLAUDE.md cadence).
- 8:00 AM safety cron re-armed this session (job 85cd754f, one-shot
  2026-07-07 08:03; session-only, dies with the terminal).
- Never point codex --output-last-message at a task's own artifact path.

## Warnings that must not be lost

- Backup risk is ACCEPTED, not solved (D-026): single-machine total loss
  remains risk #1; re-raise at every decision session.
- D-032 (TSDF codes primary) and D-037 (in-app authoring in v0.1) are
  deliberate human choices AGAINST recommendations - do not "fix" them.
- Same-day QR joiners create the fast re-ingest requirement (D-030) -
  it is S4-A acceptance, not a nice-to-have.
- The design brief's rendering numbers are labeled allocations, not
  validated measurements; DESIGN sitting (D-038) is where aesthetics get
  human eyes.
- Predecessor PII exclusion list (CLAUDE.md) applies to every Codex
  prompt AND to research tasks; nothing is cleared.
- .codex/ is gitignored scratch; session-2 interview notes live at
  .codex/interview-notes-s2.md (DECISIONS entries are the durable record).
