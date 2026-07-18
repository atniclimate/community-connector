# HANDOFF.md - Live State

> This file outranks session memory. Reading order for a new session: CLAUDE.md,
> then this file, then PLAN_1.0.md "Current Position" + "Decision Gates", then
> current-phase ADRs. DECISIONS.md is the durable judgment record (through D-048).

Last updated: 2026-07-18, one-shot build session (started as Fable 5; switched to
Opus 4.8 after an Anthropic monthly spend limit interrupted the build workflow).

## Phase status (acceptance-unit inventory, not a fraction - PLAN_1.0.md Current Position)

- **Phase 0 - Workflow/congruence/ratification foundation: COMPLETE.**
  `scripts/check-all.ps1` single orchestrator (11 members) + scoped `-Staged`
  pre-commit enforcement (D-043); AGENTS.md automated/human-only gate split;
  G-RAT/G-DATE/G-BACKUP recorded below. P0.1 (convention date) and P0.6
  (tracked-status of MANIFEST.md/PLAN_1.0.md/DEPENDENCIES.md) remain human-owned.
- **Phase 1 - Constellation legible + explore: COMPLETE except P1.3.**
  Committed and green: P1.1 troika SDF labels (offline-proven bundled OFL font);
  P1.2 focus/motion with reduced-motion variants; P1.6 legend; P1.4 search;
  P1.5 detail panel (tier code primary D-032 + governance-gated provenance
  one-liner D-033, via a cn-perm/cn-api EntityDetail extension); P1.7 flat
  reading projection (D-035 down-payment). **P1.3 benchmark re-run vs the 50ms
  p95 goal is NOT done** - park; needs the app/spike harness run + ADR-004 update.
- **Phase 2 - Persisted schemas + snapshot envelope: schemas COMPLETE; snapshot
  pipeline SCAFFOLDED + GATED + PARKED (D-048).** P2.1 op-log/export + story-path
  + snapshot-envelope schemas; P2.2 unknown-major loud rejection in readers. The
  P2.3 snapshot data-embed plugin, `--public-layer` generator, and boot reader
  (`app/src/state/snapshot.ts`, I7 unknown-major rejection) are committed but the
  embed is gated behind `CN_EMBED_SNAPSHOT=1`; default snapshot builds a valid
  single-file shell. **PARKED completion (D-048), = P2.4/P2.5:** offline
  main-thread wasm transport so the artifact carries no external worker (D-046 -
  the snapshot is NOT yet self-contained: it still emits a ~1.57MB external
  worker chunk); wire `--public-layer` into `build:snapshot`; per-artifact size
  gate; the no-leak acceptance test (D-047.4). Then flip the gate on.
- **Phase 3 - Facilitator role: role COMPLETE; wizard/forms NOT done.** P3.1-P3.4
  `GroupRole::Facilitator` with the authority matrix, fingerprint distinctness,
  and five-class no-leak property landed in Wave 1 with the mandatory adversarial
  round (D-045). **P3.5 facilitator wizard and P3.6 template-driven entry forms
  are NOT done** - park (facilitator authority already exists in cn-perm; these
  are UI-only, I2/I4 discipline).
- **Phase 4 - Stories, comprehension, snapshot acceptance: NOT STARTED.** P4.1-P4.4.
- **Phase 5 (non-gated slice):** P5.10 evidence template COMPLETE; the
  semantics-free `cn` CLI router + validate/export adapters (P5.4-partial, D-044.2)
  COMPLETE; P5.7 CPF-RCN migration recipe (docs-only) COMPLETE this session.
  P5.1-P5.3/P5.5/P5.6/P5.8/P5.9 park on G-RAT / G1 / human review as before.

## This session's arc

1. **Setup + strategy.** Codex capability interview; profiles repinned to the
   gpt-5.6-sol family (grind low / review+adversary high), full read/write granted
   (D-042). One-shot strategy put through a Codex gpt-5.6-sol adversarial round
   (CONDITIONAL NO-GO -> rulings D-044: honest status model, DP-1 = no generic
   importer, serial-integrator commit model, mechanics/language split,
   snapshot-size discipline). Facilitator blueprint at docs/blueprints/facilitator-role.md.
2. **Wave-build workflow.** Phase 0 landed green. A wave-build Workflow ran Wave 0
   (deps) + Wave 1 (labels, schemas, facilitator role, evidence template) through
   Barrier 1 (D-045/D-046), then Wave 2 lanes.
3. **Spend-limit interruption at Barrier 2.** The last three workflow agents died
   on the Anthropic monthly spend limit, leaving Wave 2 uncommitted. Switched to
   Opus 4.8.
4. **Codex-offload recovery (D-047).** Codex triage returned SALVAGE for all four
   lanes (review artifact C:\dev\_reviews\community-connector\2026-07-18_wave2-triage.md).
   The R2 detail provenance/tier core extension, R3 scoped-response fix, and boot
   reader were completed via Codex; the snapshot data pipeline was decoupled and
   parked (D-048) once Codex `exec` began persistently exiting early. Everything
   green was committed in seven atomic [codex-offload] commits (f5a913b..700f14f);
   final check-all is 11/11 green.

## Exact next actions (recommended order)

1. **Verify Codex health, then complete the snapshot pipeline (D-048 / P2.3-P2.5).**
   Add a main-thread `WasmTransport` used in snapshot mode (the `WasmClient`
   already takes a transport) so `dist/index.html` needs no external worker; wire
   `--public-layer` into `build:snapshot` (isolated non-tracked dir); extend
   `scripts/check-size.mjs` to gate every `dist/snapshot.*.html`; add the no-leak
   acceptance test (3+ above-scope sentinels absent from the HTML, D-047.4); flip
   `CN_EMBED_SNAPSHOT` on. Hardest piece - focused session, small Codex jobs.
2. **P3.5 facilitator wizard + P3.6 template-driven entry forms** (UI over
   existing cn-api submit/load; validation surfaced from cn-schema).
3. **Phase 4:** P4.1 story authoring under facilitator authority, P4.2 primer over
   the flat projection, P4.3 seeded synthetic stories + reveal-script doc, P4.4
   snapshot acceptance - all with the D-044.4 mechanics/language split
   (participant-facing prose marked "DRAFT - PENDING HUMAN REVIEW (D-023)").
4. **P1.3 benchmark** re-run with labels+halos+motion live; record in ADR-004.

## Open human gates

1. **G-RAT** - 1.0-line ratification at the plan-v3 sitting. Default: v0.1.0 is the
   committed deliverable; Phases 6-9 + the ratification-dependent Phase 5
   routing/importer specs stay provisional until ratified.
2. **G-DATE (Q-A, P0.1)** - the ATNI convention date + consent-email runway.
   Default: near-date fallback to a minimum rehearsalable pilot.
3. **G1** - vocabulary authority (ATNI Climate authors capability terms). Default:
   empty HSDS-shaped structure; fixtures stay in the two synthetic domains.
4. **G2** - Open Eligibility licensing isolated as separately-licensed data.
5. **D-023** - all community-facing text needs human review before use.
6. **License variant (Q7.4)** - PolyForm Noncommercial default; one line changes it.
7. **G-BACKUP** - single-machine loss risk ACCEPTED; re-raise each session; H:\
   mirror procedure in DEPENDENCIES.md, human-run only.
8. **P0.6** - MANIFEST.md, PLAN_1.0.md, DEPENDENCIES.md remain untracked pending
   the human's tracked-vs-review-only decision. Do NOT commit them autonomously.

## Degraded modes / standing directives

- **Codex `exec` is persistently exiting early on multi-step jobs this session**
  (the recovery, B2-finish, and the R2 adversarial round all stopped
  mid-investigation without writing output; the shorter triage completed).
  Degraded mode (CODEX_GUIDE section 8) IN EFFECT: the R2 permission commit
  (5e9c176) carries the director's own self-review and is marked
  `[unreviewed-by-codex]`. Re-verify Codex health next session; prefer small,
  bounded Codex jobs over large ones.
- Codex profiles: gpt-5.6-sol family, danger-full-access, approval never (D-042).
- Atomic per-unit commits without asking; `[codex-offload]` marks failover-mode
  commits. Verification loop before every commit; the pre-commit hook runs scoped
  check-all; a full `check-all` precedes each commit series.
- Never point codex `--output-last-message` at a task's own artifact path.

## Warnings that must not be lost

- **The snapshot is NOT yet a self-contained single file** - it still references a
  ~1.57MB external worker; an offline `file://` open will fail on worker load
  until the D-048 main-thread transport lands. `build:snapshot` ships a valid but
  data-empty shell by default (embed gated behind `CN_EMBED_SNAPSHOT=1`).
- The R2 provenance one-liner exposes the recorder's opaque PersonId (UUID, not
  PII) to all viewers - matches D-033 intent; flag to the human if undesired.
- Backup risk ACCEPTED, not solved (D-026); single-machine total loss remains
  risk #1.
- D-032 (TSDF codes primary) and D-037 (in-app authoring) are deliberate choices
  against recommendations - do not "fix" them.
- Same-day QR joiners create the fast re-ingest requirement (D-030) - S4-A
  acceptance, not a nice-to-have (parks on G-RAT).
- Predecessor PII exclusion list applies to every Codex prompt and research task.
- `.codex/` is gitignored scratch; recovery review artifacts are under
  `C:\dev\_reviews\community-connector\`.
- 8:00 AM safety cron re-armed this session (one-shot 2026-07-18 08:04,
  session-only, dies with the terminal).
