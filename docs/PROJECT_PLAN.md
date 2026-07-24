# Community Navigator - Project Plan to v0.1.0

> Durable plan. Read after CLAUDE.md. Live state stays in HANDOFF.md; this
> document changes only when the plan itself changes (log a DECISIONS entry
> when it does). Companion: docs/NEXT_SESSION.md (resume brief + the human
> interview). Execution Plan v2 accepted 2026-07-06, session 2 (D-039).
> Last revised: 2026-07-24 - reconciliation to DECISIONS.md D-050..D-056
> (gate-grill + look-back rulings; drift-check 2026-07-24). This revision
> applies recorded rulings; it makes no new decisions. Where this document
> and DECISIONS.md or HANDOFF.md disagree, the latter win.

## 1. Where the project stands

Phases 0-2 of the original phase plan are COMPLETE: contract docs and PII
tripwire; ADR-001..003 accepted through adversarial rounds; the full
Rust/WASM core (six crates, permission property test, measured 255ms fold /
133ms projection at 5k+10k); closing review with all accepted findings fixed.

Since D-039 acceptance, the item-level route lives in `PLAN_1.0.md`
(P-numbered units) and live status lives in HANDOFF.md. As of 2026-07-24:
the explore surface (SDF labels, focus/motion with reduced-motion, legend,
search, detail panel) is complete except the P1.3 benchmark; the op-log /
story / snapshot-envelope schemas and the facilitator role landed through
adversarial rounds; the R2 EntityDetail fix (D-049) is the in-progress first
unit of the reconciled route below.

The 2026-07-06 decision session (D-019..D-040) defined the pilot: an
ATNI Climate convention arc, facilitator-run, snapshot-first, with
need-to-solution routing as a hero workflow. Personal mode, identity, deep
accessibility, and governance tooling moved to v0.2. The 2026-07-24
gate-grill and look-back sessions (D-050..D-056) answered every queued gate,
pinned the pilot calendar, ratified v0.1.0, replaced the form-platform
intake assumption with the D-053 sealed-envelope architecture, and
resequenced the route for the August pilot window. Section 3 below is
Execution Plan v2 as reconciled to those rulings.

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

Note (2026-07-24): the table above records the routing as accepted with v2.
Pinned profile model ids have since moved (gpt-5.6-sol per D-042);
docs/CODEX_GUIDE.md is authoritative for current profiles and for degraded
mode when Codex misbehaves.

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

## 3. Execution Plan v2 (accepted 2026-07-06, D-039; reconciled 2026-07-24 to D-050..D-056)

> v0.1.0 = "convention-pilot ready": a facilitator-run build that takes
> intake through the facilitator pending-review queue, renders the
> participant + committee graph, routes needs to solutions, and presents
> authored stories at the ATNI Annual Convention. RATIFIED as the
> convention-arc finish line (D-052) with a sharpened acceptance bar: ready
> for actual usage across the real entity kinds (persons, organizations,
> places, skills, needs) at pilot scale (~150 expected, 300 max signups),
> not demo-grade. Personal mode, identity, deep accessibility, and
> governance tooling remain v0.2 (post-pilot). Ratification of the fuller
> 1.0 line (Phases 6-9) is deferred to a post-convention retrospective
> (D-052).

### Gate status (2026-07-24)

Answered - each by a recorded ruling, not by this document:
- **G-DATE / Q-A -> D-050:** convention 2026-09-14; internal pilots with
  several trusted groups run in August on the NORMAL app build; the recorded
  ATNI Climate collective checkpoint precedes the FIRST August ingestion.
- **G1 -> D-051:** ATNI Climate authors the capability vocabulary in its own
  words, sequenced post-stability; standard developer language over the
  HSDS-shaped structure until then (D-044.4 mechanics/language split).
- **G-RAT -> D-052:** v0.1.0 ratified as the convention-arc finish line with
  the real-usage acceptance bar above; Phases 6-9 deferred to the
  post-convention retrospective.
- **Q-B -> D-053:** NO external form platform. Intake is direct in-app entry
  plus the QR sealed-envelope relay (static GitHub Pages form + Cloudflare
  Workers/KV ciphertext relay), all landing in the facilitator
  pending-review queue; no auth in v0.1.0; CSV ingestion becomes a secondary
  path for structured sources.
- **Q-C -> D-054:** license is PolyForm Noncommercial 1.0.0; LICENSE.md
  tracked before any push.
- **P0.6 -> D-055:** PLAN_1.0.md, MANIFEST.md, DEPENDENCIES.md become
  tracked once the pre-publish world-readability sweep passes.

Still live:
- **D-023 human review** of all community-facing text (intake form, consent
  wording, tier language) before use - now concretely needed for the relay
  form and the August pilots.
- **Collective checkpoint** (D-030/D-050): a recorded ATNI Climate committee
  approval before the FIRST August internal-pilot ingestion of real people.
- **G2:** Open Eligibility mapping stays isolated if ever added.
- **G-BACKUP:** single-machine risk still ACCEPTED, not solved (D-026); the
  public remote holds code only, never data, so it is not a backup answer
  for ops.

The remote / hosting-vendor / spend gates are conditionally OPENED for
exactly one path - the public remote `atniclimate/community-connector` and
the Cloudflare Workers intake relay - with preconditions before any push or
deploy: license in-repo (satisfied), the D-055 sweep passed, core stability.
All other gates stand unchanged (CLAUDE.md gate notes).

### The pilot arc (fixed points; D-022, D-030, D-050, D-053)

1. August internal pilots: several trusted groups on the NORMAL app build on
   the pilot PC (no snapshot-pipeline dependency); D-030/D-034 consent
   process applies; the recorded collective checkpoint precedes the FIRST
   ingestion.
2. Consent text (D-023 human-reviewed) -> intake: facilitator in-app entry,
   plus QR -> static Pages form -> sealed-box encryption on the phone ->
   Cloudflare ciphertext relay -> pilot-PC pull/decrypt -> pending-review
   queue. Every submission enters the graph only on facilitator approval
   (T1 default, provenance stamped).
3. Convention 2026-09-14, general assembly: full graph reveal (the
   convention snapshot build, facilitator laptop). Early September is the
   soft deadline for pre-convention consent; most joins are expected at the
   convention itself.
4. Committee meeting: need-to-solution pathway exploration; same-day QR
   joiners appear via relay pull -> facilitator approval -> re-render.
5. Feedback capture -> post-convention retrospective (v0.2 scoping and the
   Phases 6-9 ratification decision, D-052).

### The reconciled route (D-056.3; item detail in PLAN_1.0.md, live status in HANDOFF.md)

1. **R2 EntityDetail fixes - FIRST unit (D-049; LANDED 2026-07-24).**
   The reviewer-confirmed defects, all fixed: effective tier reported over
   the PROJECTED attribute set in cn-perm (BLOCK-1); `own_settings`
   disclosure/effective-tier decisions relocated from cn-api into cn-perm
   (BLOCK-2, I2); custody/tier tests extended to all five viewer classes
   plus inactive-governance and dual-role, with an end-to-end entity_detail
   governance test (HIGH-3); control-whitespace and Unicode line-separator
   normalization in the provenance one-liner (LOW-4). The mandatory fresh
   adversarial round returned PASS-WITH-NOTES (both blockers closed; its
   residual notes were closed in the same commit).
2. **Long-lead gate-openers (parallel with 1).** The D-055 pre-publish sweep
   (update + track the three root docs; full-repo world-readability pass);
   draft ADR-005 "Remote intake: sealed-envelope relay and facilitator
   pending-review queue" (`docs/adr/ADR-005-remote-intake.md`, required
   scope in D-056.1) plus its adversarial round; intake-form/consent text
   drafted into D-023 human review; facilitator keygen ceremony design
   (offline private-key backup, key-fingerprint pinning in the puller).
3. **P3.5 facilitator wizard + P3.6 entry forms - THE intake pipeline
   (D-053).** Direct in-app entry plus the pending-review staging queue:
   every submission (in-app or remote) lands pending and enters the graph
   only on facilitator approval; queue persistence is durable-write-first,
   gitignored staging, pii-scan covered, with near-duplicate surfacing
   (D-056.4). Approved remote entries land unowned, facilitator-created
   (D-056.2); owner-binding to submitters is deferred and would trigger its
   own adversarial round.
4. **Remote intake relay (D-053, per ADR-005).** Static intake form for
   GitHub Pages (interface only; no secrets; no readable data transits or
   rests there); client-side sealed-box encryption to the facilitator public
   key (private key only on the pilot PC); minimal Cloudflare Workers + KV
   ciphertext relay (payload size caps, rate limits, KV TTL); pilot-PC
   puller that decrypts locally, stages durably into the review queue, THEN
   wipes the relay. Push/deploy only after the D-053/D-055 preconditions;
   localized offline intake stays the later stretch goal.
5. **Snapshot data pipeline (D-048) - resequenced AFTER intake; targets the
   convention build, not the August pilots.** Main-thread WasmTransport so
   the single-file artifact needs no external worker (D-046);
   `--public-layer` wired into `build:snapshot` (isolated non-tracked dir);
   per-artifact size gate; the no-leak acceptance test (D-047.4); then
   `CN_EMBED_SNAPSHOT` on by default.
6. **Phase 4 slimmed for the window (D-056.3):** minimal P4.1 story
   authoring under facilitator authority for the pilots; the rest of Phase 4
   defers, with the D-044.4 mechanics/language split throughout.
7. **P1.3 benchmark - deferred to September (post-pilot):** re-run with
   labels + halos + motion live; record against ADR-004.

Remaining to v0.1.0: the route above against the August pilot window and the
2026-09-14 convention. Live status and the next three actions are always in
HANDOFF.md, never here.

### D-039 session map - as accepted, with 2026-07-24 dispositions

The plan as accepted on 2026-07-06 sequenced the work as S3-A2 -> S3-B ->
S3-C -> Session B -> FORM -> Session C -> S4-A -> Session E -> S4-B ->
Phase 6, snapshot-first, with intake arriving as form-platform CSV exports.
That acceptance stands as history; the dispositions below revise it forward:

| D-039 item | Disposition (2026-07-24) |
|---|---|
| S3-A2 "Constellation legible" (labels, motion) | LANDED (troika SDF labels, focus/motion with reduced-motion); the benchmark re-run moved with P1.3 to September |
| S3-B "Explore + Route" (search, legend, detail, routing UI) | Explore surface landed; the detail panel carries the D-049 R2 defects - route item 1 above; remaining pieces tracked in PLAN_1.0.md |
| S3-C "Stories" + snapshot acceptance | Story authoring slims to minimal P4.1 (route item 6); snapshot acceptance rides the resequenced snapshot pipeline (route item 5) |
| Session B "Groups without engineers" (wizard, entry forms, facilitator role) | Facilitator role landed in cn-perm (adversarial round D-045); P3.5/P3.6 are promoted from UI polish to THE intake pipeline (D-053) - route item 3 |
| DESIGN sitting (D-038) | Still human-scheduled; the vocabulary/language pass is sequenced post-stability (D-051) |
| FORM "Intake form + ATNI template" | Narrows to the D-023 text draft + human review (route item 2); no form platform exists to configure (D-053); the pilot form/template design doc carries its own reconciliation banner until that draft lands |
| Session C "Dedup" -> ADR-005 | The ADR-005 slot is reassigned to remote intake (D-056.1); dedup semantics fold into the intake queue: client payload UUID + facilitator near-duplicate surfacing (D-056.4) |
| S4-A "Ingestor + CLI" | The `cn` CLI router + validate/export landed (D-044.2); CSV import becomes the SECONDARY intake path (D-053); "fast re-ingest for same-day joiners" becomes relay pull -> approve -> re-render |
| Session E "CPF-RCN migration recipe" (D-031) | LANDED docs-only (P5.7); execution stays human-gated |
| S4-B / Phase 6 close (hardening, rehearsal, tag) | Hardening + the convention rehearsal follow intake and snapshot; tag v0.1.0. "Remotes remain gated per D-026" is superseded FOR THE ONE PATH by the conditional opening (D-053/D-054/D-055); every other remote stays gated |

### v0.2 (post-pilot; shaped by feedback + the research sitting)

Personal mode (Phase 5: S5-A profile ownership/sharing, S5-B cn-sync +
protocol guide), Session D identity ADR (D-027), Session A accessibility
as primary interface (D-035), Session F sovereignty/governance tooling +
tier enforcement development (D-034), story authoring polish, and
integration spikes with the sibling tools (cap-assessor,
TCR-policy-scanner, GeoBase, engagement-database; D-040). Whether v0.2
work proceeds under the fuller 1.0 line (Phases 6-9) is the post-convention
retrospective's ratification decision (D-052).

## 4. Risk register (standing; revised 2026-07-24)

| Risk | Mitigation |
|---|---|
| Single-machine loss of ops/pilot data | Still ACCEPTED by the human (G-BACKUP / D-026). The public remote, once pushed, protects CODE only - it holds no data and is not a backup answer for ops. Re-raise at every decision session |
| August window is tight; the least compressible items are human-path | D-023 text review and the recorded collective checkpoint gate the first ingestion (D-056.4); both are framed early (route item 2) so the human can clear them before the pipeline is ready |
| Facilitator private key is a single point of total loss | Keygen ceremony design (route item 2): offline private-key backup (key material is not repo data - no G-BACKUP collision) + key-fingerprint pinning in the puller |
| Unauthenticated relay endpoint (no auth in v0.1.0) | Sealed-box ciphertext only; payload size caps, rate limits, KV TTL (ADR-005 threat model); the facilitator pending-review queue is the abuse gate; dedup via client payload UUID + near-duplicate surfacing (D-056.4) |
| Relay wipe-after-pull could lose submissions | Durable queue write BEFORE relay wipe, mandated in ADR-005 scope (D-056.1) |
| Pages form is gate-coupled to the repo's publish preconditions | The D-055 sweep is pilot-critical-path; it runs as a long-lead gate-opener (route item 2), not at the end |
| Intake response rate = graph quality | Short required core, facilitator-assisted in-app completion, QR joins at the convention (D-023, D-030, D-053) |
| Same-day joiner turnaround at the convention | Relay pull -> approval -> re-render path (route item 4) + the convention rehearsal |
| Codex sandbox bypass (D-008) | Blueprint-constrained tasks + director verification; revisit if Codex behavior ever surprises |
| Projection JSON 2.9MB per recompute at 5k scale | ADR-003 D2 typed-array escape hatch; re-measure in-browser before the convention snapshot build (snapshot work is resequenced after intake per D-048/D-056.3, so this measurement moves with it) |
| p95 tail (70ms) vs 50ms goal unvalidated | P1.3 benchmark re-run in September (D-056.3); halo mitigation is the named lever |
| "Network" circle semantics provisional | Firms up with the (human-gated) network ADR; documented in cn-perm. D-053 amends R5 for the intake path only - intake is INGEST, not sync (D-056.1) |
| Report-type duplication (cn-schema/cn-store) | Recorded debt; unify when CSV ingest lands (secondary path, D-053) |
| Session-only crons die with the terminal | HANDOFF always carries the chain; NEXT_SESSION.md carries the human-facing resume |
