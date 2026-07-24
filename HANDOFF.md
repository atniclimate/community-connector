# HANDOFF.md - Live State (pickup target)

> This file is the /pickup target and outranks session memory. Reading order for a
> fresh session: CLAUDE.md, then this file, then PLAN_1.0.md "Current Position" +
> "Decision Gates", then current-phase ADRs. DECISIONS.md is the durable judgment
> record (through D-058; D-057/D-058 are the 2026-07-24 R2-fix and sweep-unit
> records). Every path below was verified on disk at the 2026-07-24 close.

## What this project is

Community Navigator (repo folder `community-connector`) is a local-first, privacy-first
tool for a community to see itself as a permission-filtered 3D graph of people, places,
orgs, skills, and needs, and to route a need to people/resources who can meet it. A Rust
core compiled to WASM owns data, schema, permissions, and graph queries; the TypeScript
app renders only viewer-scoped projections it is handed (it never re-implements
permission logic). The committed near-term deliverable is **v0.1.0 "convention-pilot
ready"** (ratified D-052): a facilitator-run build for the ATNI Climate convention arc -
August internal pilots on the normal app build, convention 2026-09-14.

## Where everything lives

| What | Path |
|---|---|
| Durable contract (mission, R1-R10, gates, autonomy) | `CLAUDE.md` |
| Invariants I1-I12 (review standard) | `AGENTS.md` |
| Decision register (D-001..D-058) | `DECISIONS.md` |
| Execution Plan v2 (accepted D-039; reconciled 2026-07-24) | `docs/PROJECT_PLAN.md` section 3 |
| Route map to 1.0 (v1.1, TRACKED per D-055/D-058) | `PLAN_1.0.md` |
| Repo inventory snapshot (descriptive, 2026-07-24) | `MANIFEST.md` |
| External-dependency audit (world-readable revision) | `DEPENDENCIES.md` (+ machine-local detail in gitignored `_private/`) |
| Accepted architecture decisions | `docs/adr/` (ADR-001..004; ADR-005 remote intake DRAFT pending adversarial round) |
| Intake/consent text package (DRAFT, pending D-023) | `docs/design/intake-consent-text-draft-2026-07-24.md` |
| Facilitator keygen ceremony design | `docs/design/facilitator-keygen-ceremony.md` |
| Facilitator role blueprint / authority matrix | `docs/blueprints/facilitator-role.md`, `docs/design/authority-matrix.md` |
| Snapshot scope + byte ledger | `docs/design/snapshot-viewer-scope.md`, `docs/design/snapshot-ledger.md` |
| Pilot FORM draft / evidence template / migration recipe | `docs/design/pilot-form-and-template-2026-07-06.md`, `docs/pilot-evidence-template.md`, `docs/cpf-rcn-migration-recipe.md` |
| Verification battery (11 members) | `scripts/check-all.ps1` (+ `scripts/hooks/pre-commit`) |
| Rust core | `core/crates/cn-{model,schema,store,perm,graph,api,wasm}`, `core/cli` (`cn`) |
| App | `app/src/{viz,ui,state,wasm,theme}`; snapshot boot reader `app/src/state/snapshot.ts` |
| Codex adversarial artifacts | out-of-repo review lane `_reviews/community-connector/` under the dev workspace |
| Prior handoffs | `docs/archive/handoffs/` |

## State of play

**DONE and verified (check-all 11/11 green at every 2026-07-24 commit; clean tree):**
- Phase 0: `check-all` orchestrator + scoped pre-commit enforcement (D-043); gates recorded.
- Phase 1 explore surface (COMPLETE except P1.3, deferred to September): troika SDF labels,
  focus/motion with reduced-motion, legend, search, detail panel, flat reading projection.
- Phase 2 schemas: op-log/export + story + snapshot-envelope schemas; unknown-major loud
  rejection (P2.1/P2.2).
- Phase 3 facilitator role: `GroupRole::Facilitator` + authority matrix + fingerprint
  distinctness + five-class no-leak property (P3.1-P3.4, adversarial round D-045).
- Phase 5 slice: semantics-free `cn` CLI router + validate/export (P5.4-partial, D-044.2);
  pilot evidence template (P5.10); CPF-RCN migration recipe (P5.7, docs-only).
- **R2 EntityDetail fixes (D-049 -> D-057, commit 73b5656):** effective tier over the
  projected attribute set in cn-perm; own_settings deleted, cn-api now a pure carrier
  (I2); full viewer-class custody+tier test matrix incl. e2e; one-line normalization
  incl. U+2028/U+2029. Fresh adversarial round PASS-WITH-NOTES, both blockers CLOSED,
  residual notes closed in the same commit. D-049 discharged.
- **Gate-opener drafts (commit 7afc20e):** ADR-005 "Remote intake" DRAFT (full D-056.1
  scope) awaiting its adversarial round; D-023 consent-text package (NOT FOR USE until
  human review); facilitator keygen ceremony design.
- **D-055 sweep unit (D-058, commit 8a58c7f):** six-agent world-readability scan +
  revisions; `_private/` split for machine-local content; PLAN_1.0.md v1.1, MANIFEST.md,
  DEPENDENCIES.md revised and TRACKED; PROJECT_PLAN.md full D-056.5 revision. Six
  needs-human dispositions remain (D-058) - the first-push precondition is not yet met.

**NOT done - ordered next actions:**
1. **ADR-005 adversarial round.** The draft is committed; run the mandatory round
   (adversary wrapper), judge findings, amend, mark ACCEPTED. Required before any relay
   implementation or deploy.
2. **P3.5 facilitator wizard + P3.6 entry forms - THE intake pipeline (D-053).**
   Direct in-app entry plus the pending-review staging queue: every submission (in-app
   or remote) lands pending and enters the graph only on facilitator approval; queue
   persistence is durable-write-first, gitignored staging, pii-scan covered, with
   near-duplicate surfacing (D-056.4). UI over existing cn-api submit/load; validation
   surfaced from cn-schema; I2/I4. Approved remote entries land unowned,
   facilitator-created (D-056.2). Queue format per ADR-005 D4.
3. **Remote intake relay implementation (per accepted ADR-005).** Static Pages form,
   client-side sealed-box encryption, Workers+KV ciphertext relay, pilot-PC puller with
   fingerprint pinning (keygen ceremony doc). Push/deploy prereqs: license in-repo
   (satisfied), D-058 needs-human dispositions resolved, core stability.
4. **Snapshot data pipeline (D-048 / P2.3-P2.5) - targets the convention build.**
   Main-thread `WasmTransport` (D-046); `--public-layer` into `build:snapshot`;
   per-artifact size gate; no-leak acceptance test (D-047.4); then flip
   `CN_EMBED_SNAPSHOT` on by default. Hardest piece.
5. **Phase 4 slimmed (D-056.3):** minimal P4.1 story authoring under facilitator
   authority for the pilots; P4.2/P4.3 bulk defer; P4.4 rides item 4.
6. **P1.3 benchmark** deferred to September (post-pilot); record in ADR-004.

## The human's queue

1. **D-058 pre-push dispositions (six items, see DECISIONS.md D-058):** maintainer
   emails in D-003/D-011 + pii-allowlist; predecessor-repo PII disclosure wording in
   CLAUDE.md / LAUNCH_PROMPT.md / migration recipe; THE_STORY.md public approval;
   pilot-form Parts A/C draft-in-public decision; workspace-path acceptance
   (recommended accept); mirror-carries-_private confirmation (recommended yes).
   The first push to `atniclimate/community-connector` waits on these.
2. **Collective checkpoint:** a recorded ATNI Climate committee approval must exist
   BEFORE the first internal-pilot ingestion of real people (D-030/D-050).
3. **D-023 review:** a concrete draft now exists to review -
   `docs/design/intake-consent-text-draft-2026-07-24.md` (form fields, consent
   statement, consent email, QR explainer, reviewer checklist). Also carries the
   ADR-005 open question on rejected-record retention (a consent-semantics call).
4. Standing: Open Eligibility mapping isolated if ever added (G2); single-machine
   backup risk still ACCEPTED, not solved (G-BACKUP / D-026) - the public remote
   will hold code only, never data, so it is not a backup answer for ops.

## Non-negotiables a fresh session must not violate

- **Human gates are absolute** (CLAUDE.md). Three were OPENED by the human on
  2026-07-24 with conditions (D-053/D-054/D-055): the public remote
  `atniclimate/community-connector` may be added and pushed ONLY after the
  license is in-repo (satisfied), the pre-publish sweep has passed (sweep RAN,
  D-058 needs-human items OUTSTANDING - so not yet), and core stability is
  reached; Cloudflare Workers spend is approved for the intake relay only.
  Everything else stands unchanged: no real-person PII in the repo, any commit,
  any fixture, or any Codex prompt (I1); no other spend; no protocol/identity/
  federation commitment beyond D-053's intake path.
- Never commit `_private/` content; it is the sweep's machine-local landing zone.
- Permission logic lives only in `cn-perm` (I2); state mutates only through `app/src/state`
  (I4); every entity/edge carries provenance + tier (I6); persisted formats are versioned
  with unknown-major rejection (I7); snapshot stays under 5MB (I8); docs use hyphens (I10).
- Verification loop before every commit; atomic per-unit conventional commits; the
  pre-commit hook runs scoped `check-all`, a full `check-all` precedes each commit series.
- Permission-adjacent work gets a director blueprint + a mandatory adversarial round.
  ADR-005 is NOT accepted until its round runs.

## Key design commitments (shortest refresher)

- Rust/WASM core is the single source of truth; the app renders permission-filtered
  projections only. Event-sourced op log in `cn-store`; state is a fold over ops.
- Intake (D-053, ADR-005 draft): in-app entry + facilitator pending-review queue is
  the primary path; remote path is QR -> static Pages form -> client-side sealed-box
  encryption -> Cloudflare ciphertext relay -> pilot-PC pull/decrypt/durable-stage ->
  relay wipe. Queue lives OUTSIDE the op log. No auth in v0.1.0/v1.0. No server ever
  holds readable personal data; the graph never listens on the network. Approved
  remote entries land unowned, facilitator-created (D-056.2).
- Snapshot-first delivery (D-024) now targets the CONVENTION build (D-056.3); August
  pilots run the normal app build. The snapshot is NOT yet self-contained (still emits
  a ~1.57MB external worker) - that is next-action #4.
- TSDF tier codes primary in the UI (D-032); in-app story authoring in v0.1 (D-037) - both
  deliberate choices against recommendations; do not "fix" them.
- Codex offload (gpt-5.6-sol, D-042): the adversary wrapper ran two clean rounds on
  2026-07-24 (R2 fix review completed in ~6 min with receipt). The
  [[codex-exec-early-exit]] caution stands for long raw `codex exec` jobs; the wrapper
  path is currently healthy.
