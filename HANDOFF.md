# HANDOFF.md - Live State (pickup target)

> The /pickup target; outranks session memory. Reading order for a fresh session:
> `CLAUDE.md`, this file, `docs/NEXT_SESSION.md` (launch cards), then
> `docs/planning/NEXT-PHASE-convention-e2e.md` and `SESSION_ROSTER.yaml` for the phase
> being worked. Written at the 2026-09-12 evening true-up. Previous handoff:
> `docs/archive/handoffs/2026-09-12-pickup-director.md`.
>
> **The repo is PUBLIC** (origin `https://github.com/atniclimate/community-connector`).
> Local `main` is **70 commits ahead of origin and UNPUSHED** (`git rev-list --count
> origin/main..main` = 70 before this handoff's own commit). Pushing is permitted by
> CLAUDE.md's gate notes but has not been run this arc; ask the human first.

## What this project is

Community Navigator (repo folder `community-connector`) is a local-first, privacy-first
tool for a community to see itself as a permission-filtered 3D graph of people,
committees, organizations, skills, and needs, and to route a need to people who can meet
it. A Rust core compiled to WASM owns data, schema, permissions, and graph queries; the
TypeScript app renders only the viewer-scoped projections it is handed. The near-term
deliverable is v0.1.0 "convention-pilot ready" (D-052) for the ATNI convention on
**2026-09-14**: a presenter-mode reveal of the convention community on the reference
laptop, running on synthetic data unless the human clears the go-live gates.

## Where everything lives

| What | Path |
|---|---|
| Durable contract (mission, R1-R10, gates, autonomy) | `CLAUDE.md` |
| Invariants I1-I12 (review standard) | `AGENTS.md` |
| Decision register (D-001..D-099) | `DECISIONS.md` |
| Session plan of record (R-sessions, next-phase S-E1..S-E5) | `SESSION_ROSTER.yaml` |
| Next-phase plan: go-live checklist, research receipts, sessions, wow slice W1-W5 | `docs/planning/NEXT-PHASE-convention-e2e.md` |
| Route map to 1.0 (P-ids) and task trace | `PLAN_1.0.md`, `TRACE.yaml` |
| 2026-09-12 bootstrap reconciliation (deploy-bar table section 5B) | `docs/planning/RECONCILIATION-2026-09-12.md` |
| Discovery memo (candidate decisions D-090c..D-096c) | `docs/research/discovery-2026-09-12.md` |
| Execution plan v2, repo inventory, dependency audit | `docs/PROJECT_PLAN.md`, `MANIFEST.md`, `DEPENDENCIES.md` |
| Accepted ADRs (001-005) | `docs/adr/` |
| Blueprints (intake pipeline, relay, presenter mode, facilitator role, ...) | `docs/blueprints/` |
| Runbooks (live entry, remote-intake e2e, relay deploy DRAFT) | `docs/runbooks/` |
| Consent text DRAFT (pending D-023), keygen ceremony design | `docs/design/intake-consent-text-draft-2026-07-24.md`, `docs/design/facilitator-keygen-ceremony.md` |
| Reveal launcher and presenter controls reference | `scripts/reveal.ps1`, `CONTROLS.md` |
| Presenter beats (plain JSON array) | `app/public/beats.atni.json` |
| ATNI convention synthetic template and ops | `fixtures/templates/atni-convention.template.json`, `fixtures/groups/atni-convention.ops.jsonl` (generator `app/scripts/generate-atni-ops.mjs`) |
| Verification battery (12 members) and pre-commit hook | `scripts/check-all.ps1`, `scripts/hooks/pre-commit` |
| Rust core | `core/crates/cn-{model,schema,store,perm,graph,api,wasm,ingest,sync}`, `core/cli` (`cn`, incl. `intake/`) |
| Remote intake (built, accepted D-089, not deployed) | `form/`, `relay/`, `core/cli/src/intake/`, e2e `scripts/e2e-remote-intake.ps1` + `scripts/e2e/` |
| App (renderer `viz/`, hover overlay `viz/hover.ts`) | `app/src/{viz,ui,state,wasm,theme}` |
| Agent definitions | `.claude/agents/scout.md`, `.claude/agents/coordinator.md` |
| Archived handoffs and executed plans | `docs/archive/handoffs/`, `docs/archive/plans/` |
| Codex review lane (off-repo) | `C:\dev\_reviews\community-connector\` (latest: `2026-09-12_threejs-viz-critique.md`, `2026-09-12_viz-quickwins-diff-review.md`) |

## State of play

**DONE (verified at HEAD; last full `pwsh scripts/check-all.ps1` = 12/12 PASS at
`10dbc78`; only docs commits since):**
- Everything through the remote-intake relay (steps 1-11, D-089 ACCEPT-WITH-FIXES):
  unchanged, see the archived handoffs.
- Reveal chain 2026-09-12: S-R2 ATNI template + synthetic fixture (60 people, 15
  committees, 12 orgs); S-R4a cn-graph measures + `graph_measures`; S-R4b presenter mode
  (status `partial` - human visual gate open); S-R7 `scripts/reveal.ps1` + `CONTROLS.md`.
- S-R0 partial: the clippy fix `7be2b5b` landed. See NOT done.
- Viz quick wins (D-097, `81af3f2..bef39df`): capped DPR and idle render loop; fog and
  color pipeline for edges and halos; clustered layout; halo culling plus selection ring;
  hover tooltip; fit on load; label size cap; presenter layout fix.
- Convention reveal (D-098, `0cb27f0..290820d`): `?group=atni-convention` (governance
  viewer sees all 87 entities); frustum fit with caption inset; kind beats foreground
  their kinds; emphasized labels; camera-facing committee rings. Codex diff review
  (changes requested) dispositioned in `290820d`: stale hover cache and drift-wait
  rendering fixed (browser-measured: 0 frames while idle), plus the Medium items;
  deferrals recorded in D-098.
- Evening true-up: next-phase plan + S-E1..S-E5 (D-099); archive moves; CLAUDE.md's
  stale renderer and crate-list lines fixed.

**NOT done - ordered next actions:**
1. **S-R0 remainder** (tiny): add `rust-toolchain.toml` (none exists) and move
   `troika-three-text` from devDependencies to dependencies in `app/package.json`.
2. **S-E3 presenter controls first** (director ordering for the 2026-09-14 date; the plan
   lists E1 first): `c`/`o` (and `m`) kind keys plus an on-screen rail, `t` exact-match
   tribe beat, `s` per D-099c, with W1 and the W3 `nameOnStage` name gate folded in.
   Needs no schema or durable-owner change. Human visual gate afterward.
3. **Codex review of `0cb27f0..d0e15a0`** (after the reviewed range; still
   `[unreviewed-by-codex]`), then of S-E3.
4. **S-E1** ATNI form build switch + theme-token slot (placeholder palette until the
   human supplies ATNI assets).
5. **S-E2** intake-to-edges (an approval emits one `EntityCreate` and no edges today,
   `core/crates/cn-ingest/src/approval.rs:602-624`); permission-adjacent, mandatory
   adversarial round, verdict recorded by the human.
6. **S-E4** scripted end-to-end rehearsal on synthetic data (real browser, local
   `wrangler dev`, pull, wizard approve, apply, reload, assert node + edges); closes the
   D-089-owed real-browser form smoke.
7. **S-E5 / S-R1** deploy-bar engineering (Pages workflow, D8 hash step, keygen
   rehearsal), non-executing until the human clears the gates. Later: W2/W4/W5, S-R3, S-R5,
   S-R6, maintenance chain.
- Known unverified: `relay/` and `form/` vitest suites (47/47 each at D-089) have not
  been re-run since 2026-08-11; they are not check-all members.

## The human's queue

1. **Visual sign-off on this laptop's real GPU** (S-R4b gate): `pwsh scripts/reveal.ps1`,
   click Present, step the beats. All viz evidence so far is SwiftShader screenshots.
2. **ATNI brand assets** for the dark-mode form and app: palette values, font files, and
   their licenses (S-E1 uses placeholders until then).
3. **D-099c**: what `s` means on the presenter keyboard; later, whether `tribe` stays
   free text or becomes a fixed vocabulary.
4. **Go live at the convention or not** (D-090c, candidate). Live requires every row of
   the go-live checklist (`docs/planning/NEXT-PHASE-convention-e2e.md` section 1): D-023
   sign-off on the consent/form text; the real keygen ceremony; Pages source set to
   "GitHub Actions"; the Workers tier (D-092c, spend); and the RECORDED ATNI Climate
   committee approval before any real ingestion - plus the engineering rows.
5. **Record, amend, or reject D-090c..D-096c** (discovery memo, last section). D-094c
   (template shape) is already realized in the synthetic fixture; D-096c (toolchain pin)
   is half done.
6. **Push**: 70 commits are local only. Say when.
7. Standing: G-BACKUP accepted not solved; pilot-window close needs the recorded
   rejected-record purge sweep (D-059.11).

## Non-negotiables a fresh session must not violate

- **Public repo, privacy first (I1):** no real-person PII in any file, commit, fixture,
  or Codex prompt; never commit `_private/`. On the projector, show counts, not
  individual names, until an on-stage consent line exists (D-099).
- **Deploy bar D-059.8 is UNMET:** nothing goes live on Pages or Workers. No real
  ingestion before the recorded committee checkpoint. Cloudflare spend only for the
  relay, and the tier is a human call.
- **The graph never listens (ADR-005):** new entries reach the app only by a facilitator
  action (apply, then reload); no polling, sockets, or file watches.
- Permission logic only in `cn-perm` (I2); app state mutates only through `app/src/state`
  (I4); provenance + tier on everything (I6); versioned formats (I7); snapshot under 5MB
  (I8); hyphens in docs (I10); reduced-motion variant for every motion (I9).
- ADR-004 renderer stands (owned instanced Three.js; no bloom, particles, second edge
  system, or new runtime dependency without a recorded decision). Standing rulings no
  session may "fix": D-032, D-037, D-051, D-056.4.
- Verification loop before every commit; atomic conventional commits; permission-adjacent
  work (S-E2, S-R3) gets a blueprint and a mandatory adversarial round.
- Shared-checkout hygiene: stage and commit only your own paths in one breath (two
  sessions sharing one index swept each other's files on 2026-09-12).

## Key design commitments (shortest refresher)

- Rust/WASM core is the single source of truth; the app renders permission-filtered
  projections only; the op log is event-sourced and state is a fold over it.
- Intake: in-app facilitator entry is primary; remote path is QR -> Pages form ->
  client-side sealed box -> Cloudflare ciphertext relay -> facilitator PC pulls,
  decrypts, stages -> facilitator approves -> `cn intake apply` is the only writer.
- Presenter mode: beats from `app/public/beats.atni.json`; kind beats dim non-matching
  entities; measure beats use `graph_measures` (betweenness, single-tie); fits keep the
  caption clear; Escape exits.
- TSDF tier codes primary in the UI (D-032); in-app story authoring in v0.1 (D-037).
