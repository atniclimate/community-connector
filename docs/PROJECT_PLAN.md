# Community Navigator - Project Plan to v0.1.0

> Durable plan. Read after CLAUDE.md. Live state stays in HANDOFF.md; this
> document changes only when the plan itself changes (log a DECISIONS entry
> when it does). Companion: docs/NEXT_SESSION.md (resume brief + the human
> interview). Last revised: 2026-07-06, end of session 1.

## 1. Where the project stands

Phases 0-2 are COMPLETE: contract docs and PII tripwire; ADR-001..003
accepted through adversarial rounds; the full Rust/WASM core (six crates,
permission property test, measured 255ms fold / 133ms projection at
5k+10k); closing review with all accepted findings fixed.

Phase 3 is roughly HALF done: ADR-004 renderer decision measured on the
reference GPU; design brief final; state machine (I4); theming pipeline
with CVD-clean fixture palettes; base renderer landed and visually
verified (first screenshot: docs/design/screenshots/). Remaining Phase 3
work and Phases 4-6 are planned below.

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

## 3. Plan by phase (explicit sessions)

### Phase 3 remainder - Frontend (3-4 build sessions + 2 deep sessions)

**S3-A "Constellation visible"** (next build session)
- Fix the visual-check findings: node materials not receiving kind colors
  (bodies render near-black), edge/node visual hierarchy inverted, halo
  culling too aggressive, background gradient presence. (grind medium,
  Fable re-screenshots to accept.)
- Labels: troika SDF, zoom-adaptive policy, density culling, cap tokens.
  (grind HIGH - fiddly.)
- Motion: focus-mode dim/highlight via the dual color buffers, camera
  fly-to polish, idle drift, all with reduced-motion variants. (grind
  medium from brief section 3.)
- Acceptance: screenshot review by Fable; spike benchmark re-run with
  labels+halos live (record vs the p95 <= 50ms tail goal).

**S3-B "Explore"**
- View modes from template data; detail panel (entity_detail path,
  own-record indicators); search UI over cn-graph search; legend with the
  "adjusted for readability" indicator. (blueprints Fable; grind medium.)

**Session A (deep) "Accessibility as the primary interface"**
- The parallel DOM list/table is the PRIMARY equivalent interface (brief
  ruling; WCAG 2.1.1/4.1.3). Design pass first: information architecture,
  focus model, canvas<->DOM sync through the store, 375px layouts.
- Interview input needed (see NEXT_SESSION Q6) - real AT users, elder
  accessibility, language.
- Split: codex review researches WAI-ARIA APG patterns (memo, capped);
  Fable designs; two grind tasks (DOM interface; keyboard/announcements).

**Session B (deep) "Groups without engineers"**
- Group setup wizard + template-driven entry forms (R1/R2/R3 UI): field
  widgets per attribute type, validation UX from cn-schema findings,
  drafts. Interview input needed (Q7): what group creation feels like for
  a non-technical committee admin; paper-flow the wizard first.

**S3-C "Stories + close"**
- Story viewing UI (validated data, silent elision already in core).
- Snapshot acceptance: embed a projection + theme; both fixture groups
  load and render; size budget green.
- R9 audit (acceptance session) + Phase 3 closing codex review sweep +
  rulings + fixes. Phase 3 CLOSED.

### Phase 4 - Ingestor (2 build sessions + 2 deep sessions)

**Session C (deep) "Who is the same person?"** - dedup/entity-resolution
design -> ADR-005: match heuristics, merge-vs-link ops, undo, review
queues. Interview input (Q8): default to manual merge review? Then grind
implements against fixtures with planted near-duplicates.

**S4-A** - cn-ingest importers (CSV/JSON/YAML -> ops with provenance +
tier), cn CLI subcommands (ingest, validate, export, snapshot), fixture
round-trip lossless. Mostly grind from blueprints; the CLI reuses cn-api.

**Session E (deep, docs-only) "CPF-RCN migration recipe"** - the recipe
the human executes: what to export from the old repo, scrub steps, tier
assignment per field, FPIC checkpoints. The session NEVER reads red data.
Interview input (Q4-Q5) required before this session is scheduled.

**S4-B** - Phase 4 closing review + fixes. Phase 4 CLOSED.

### Phase 5 - Personal mode (2-3 sessions + 2 deep sessions)

**Session D (deep) "Identity"** - the ADR-003 declared dependency:
core-owned sessions, local credentials, how trust grants bind to
identities that could later federate. STARTS from interview answers (Q3);
ends with an ADR draft + any residual gate questions. Nothing in personal
mode ships before this ADR is accepted.

**S5-A** - profile ownership + per-attribute sharing UI + trust grant
management + grant audit log + viewer-context switcher ("view as").
Blueprints Fable (permission-adjacent = grind HIGH; heavy adversarial
review after).

**S5-B** - cn-sync trait finalized + LocalLoopback + the protocol
integration guide for a future network team (Fable-heavy writing; codex
drafts from ADR-002 A-B8). Session F folds in here or stands alone:

**Session F (deep) "Sovereignty made visible"** - export/governance UX:
which viewer contexts are legitimate export targets, tier language in the
UI, governance action flows (TierSet), CARE principles as product
behavior. Interview input (Q5) is the authority here.

### Phase 6 - Hardening -> v0.1.0 (1-2 sessions)

- Full-stack perf pass at 2-5k (benchmark re-run; validate/revise the
  tail goal; ADR-004 table updated).
- Error-state UX sweep; empty states; loading states.
- Docs: group-admin guide + individual guide (codex drafts, Fable voice
  pass, HUMAN reviews community-facing language before they count).
- Full codex review sweep over everything since Phase 2 close -> rulings
  -> fixes -> tag v0.1.0 locally (remotes remain gated).

Estimated remaining: 10-13 focused sessions.

## 4. Risk register (standing)

| Risk | Mitigation |
|---|---|
| Single-machine repo: disk failure loses everything | GATE QUESTION Q1 (private remote) - highest-priority interview item |
| Codex sandbox bypass (D-008) | Blueprint-constrained tasks + director verification; revisit if Codex behavior ever surprises |
| Projection JSON 2.9MB per recompute at 5k scale | ADR-003 D2 typed-array escape hatch; re-measure in-browser before Phase 6 |
| p95 tail (70ms) vs 50ms goal unvalidated | S3-A benchmark re-run; halo mitigation is the named lever |
| "Network" circle semantics provisional | Firms up with the (human-gated) network ADR; documented in cn-perm |
| Report-type duplication (cn-schema/cn-store) | Recorded debt; unify when cn-ingest lands (S4-A) |
| Session-only crons die with the terminal | HANDOFF always carries the chain; NEXT_SESSION.md carries the human-facing resume |
