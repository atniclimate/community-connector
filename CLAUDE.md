# Community Navigator - Contract Document

> Durable contract for every Claude Code session on this project. Read this first,
> then `HANDOFF.md` (live state, outranks memory), then `docs/PROJECT_PLAN.md`
> (the session-by-session plan and Fable/Codex routing model), then the current
> phase's ADRs in `docs/adr/`. When the human is present at session start, also
> read `docs/NEXT_SESSION.md` and offer them its brief and interview. The full
> founding brief is archived at `docs/LAUNCH_PROMPT.md`.

Product name: **Community Navigator**. Repo folder: `community-connector` (mandated;
see DECISIONS.md D-001). All docs use hyphens, never em dashes.

---

## Mission

Community Navigator is a reusable tool for communities (ATNI committees, working
groups, research teams, and others) to see themselves as a living 3D graph of people,
places, organizations, skills, and needs, so that a problem or need can be visually
and computationally connected to a solution.

## Prime directives (strict priority order)

1. **Privacy first.** No real person's PII (names paired with emails, personal contact
   info, unreleased affiliations) ever enters this repository, any commit, any fixture,
   or any Codex prompt. This outranks every feature goal.
2. **Human gates are absolute** (see Gates below). Never answered autonomously.
3. **Maximize autonomous progress inside those bounds.** Make reversible decisions,
   log them in `DECISIONS.md`, keep moving. Park blocked work; switch tracks rather
   than idling.
4. **Foundations over features.** Domain model, permission engine, and protocol
   abstraction get the deep-thinking ladder; UI polish gets speed.

## Requirements (traceability anchors)

- **R1** Multi-group: groups are first-class; new-group setup is a workflow, not a fork.
- **R2** Extensible attributes: entity kinds/attributes defined per group by schema
  templates; typed attributes (text, number, enum, tags, date, geo, link, media ref).
- **R3** Data entry and ingestion: in-app UI plus a structured-source ingestor
  (CSV/JSON/YAML) with dedup, provenance stamping, validation.
- **R4** Personal mode: individuals own their record; per-attribute sharing across
  visibility circles: private, trusted, group, network, public. Every view of the
  graph is a permission-filtered projection for that viewer.
- **R5** Network-ready, not networked: stable UUIDv7 IDs, append-only op log as the
  unit of exchange, `SyncTransport` abstraction with local-only adapter. No actual
  networking; protocol/identity-standard choice is a human gate.
- **R6** Graph core: filtering, search, typed traversals, shortest/constrained paths
  (need-to-solution routing), neighborhood expansion, permission-aware projection;
  fast for low-thousands of nodes in the browser.
- **R7** Stories: curated narrative paths, authored in-app or via data files,
  validated at load.
- **R8** Distribution: offline single-file HTML snapshot (5MB budget) plus a normal
  app build.
- **R9** Accessibility and responsiveness from the start: ARIA, keyboard navigation,
  `prefers-reduced-motion`, 375px layouts - Phase 3 acceptance criteria.
- **R10** Provenance and tiering: IEEE 2890-style provenance envelope and a
  data-sensitivity tier on every entity and edge; permission engine and exports
  respect tiers.

## Architecture stances (change only via ADR plus one adversarial Codex round)

- **Rust core compiled to WebAssembly** (wasm-bindgen/wasm-pack) is the single source
  of truth for data, schema validation, permissions, and graph queries. Same crates
  power the native `cn` CLI and ingestor. The frontend never re-implements permission
  logic; it renders projections the core hands it.
- **Rendering stays in TypeScript** (3d-force-graph + Three.js to start). The core
  does no layout or rendering; replacing the renderer must not touch the core.
- **Permission-filtered projection is the only read path.** Every graph the app
  receives is computed by `cn-perm` for a viewer context (anonymous, group member,
  trusted peer, self, admin). There is no unfiltered client API.
- **Event-sourced change log in `cn-store`.** Mutations are appended operations;
  state is a fold over them. This is the R5 network-readiness bet. ADR-002 covers it,
  including rejected alternatives (CRDTs now, snapshot-only now).
- **Group templates are data**, validated by `cn-schema` against
  `schemas/group-template.schema.json`. The UI renders entry forms from templates.
  A new community type ships as a JSON file, not code.
- **Stories are data**, validated at load by the core, referencing entities by
  stable ID.

## Repo layout

```
CLAUDE.md AGENTS.md HANDOFF.md DECISIONS.md
docs/            CODEX_GUIDE.md, LAUNCH_PROMPT.md, ENVIRONMENT.md, adr/, design/
schemas/         JSON Schemas: group templates, ingest formats, Codex output
                 contracts, story paths
core/            Rust workspace
  crates/        cn-model cn-schema cn-perm cn-graph cn-store cn-sync cn-ingest cn-wasm
  cli/           cn CLI: ingest, validate, export, snapshot build
app/             TypeScript strict + Vite; src/viz src/state src/ui src/wasm
fixtures/        SYNTHETIC data only; two contrasting demo groups minimum
scripts/         pii-scan, build orchestration, checks
```

## Invariants

`AGENTS.md` holds the canonical I1-I12 checklist. It is the review standard for every
diff, Codex or human. Do not restate it elsewhere; link to it.

## Human gates - never decide, always park and escalate in HANDOFF.md

- Creating or adding any git remote; publishing or sharing anything outside the machine.
- Ingesting, transcribing, or fixturing any real person's data.
- Spending money: new paid services, APIs, or account upgrades.
- Committing to an external network protocol, identity standard, federation, or
  hosting vendor (designing the abstraction is ours; choosing the network is not).
- Choosing the project license.
- Breaking schema changes after v1.0 formats exist.
- Anything involving the old repo's (`C:\dev\CPF-RCN_demo`) git history or its PII
  remediation.

### Real-data gate process - ATNI convention pilot (D-030, D-034)

- Individual consent instrument: the intake form. Only form respondents enter the
  graph; QR-code joins at the convention are consented joins. The convention
  attendee list is outreach-only data and is never ingested or rendered.
- Collective checkpoint: a recorded ATNI Climate committee approval of the network
  activity before any real ingestion runs.
- All pilot entries and outputs enter at TSDF T1. The tier-assignment authority is
  ATNI Climate. Per-field tier UX is post-pilot work.
- Community-facing text (intake form, consent email, tier wording) requires human
  review before use.
- Real pilot data lives outside this repository (or in gitignored staging) and is
  never committed; the PII prime directive is unchanged by this process.

## Yours to decide autonomously (log nontrivial ones in DECISIONS.md)

Crate and library choices within the pinned stack; internal APIs and module
boundaries; test strategy; schema drafts while versions are 0.x; UI/UX design;
fixture content; refactors; Codex task routing; phase-internal sequencing.

## Predecessor repo rules

`C:\dev\CPF-RCN_demo` is read-only reference. Port concepts and isolated frontend
techniques; never bulk-copy code; never copy data. Hard PII exclusion list (never
read, copy, fixture, or pipe to Codex): `source_data/` (all), `T1_partners/`,
`research_edges/`, `entities/persons/*.yaml`, `archive/MANIFEST.md`,
`archive/vision_docs/data_participants.txt`, `data/*.json` outputs derived from
those, and any file pairing a real name with contact info.

## Autonomy protocol

- **Stop the line** (halt, record in HANDOFF.md, switch tracks) if: PII is found in
  anything about to be committed or sent to Codex; contract docs contradict each
  other; the toolchain breaks beyond two focused repair attempts; a human gate is
  about to be crossed for momentum.
- **Deep-thinking ladder** (docs/CODEX_GUIDE.md section 4): patch, root cause,
  research, redesign. Never skip rungs downward; timebox rungs 2-3 to one focused
  pass each; every climb past rung 1 gets a DECISIONS.md entry (trigger, causal
  chain, options, choice, strongest surviving objection). Phase 1 is presumptively
  rung 3-4 work.
- **Verification loop before every commit.** Rust: `cargo fmt --check`,
  `cargo clippy -- -D warnings`, `cargo test`. App: `tsc --noEmit`, `vite build`,
  fixture smoke load. Cross-cutting: PII scan; size budget check when the snapshot
  build changed. Then Codex review pass, then judgment, then a conventional commit.
  Small commits; never batch a day's work into one.
- **Cadence:** many small completed units over one heroic branch. Report Codex usage
  at ~50% of session budget. A 2x estimate blowout is a ladder trigger, not a reason
  to grind harder.
- **Atomic writes and commits (human directive, 2026-07-06):** every completed,
  verified unit commits immediately as its own conventional commit - never ask
  permission to commit, never batch units, never leave a verified unit sitting
  uncommitted. Files are written whole (no partial staged states between units).
- **Token routing (human directive, 2026-07-06):** the routing policy in
  docs/CODEX_GUIDE.md section 7 is mandatory practice, not advice: Codex absorbs
  research, first drafts, mechanical implementation, and bulk transforms; Claude
  reads compressed outputs, judges, and commits. Re-arm a one-shot 8:00 AM local
  resume cron each session while this directive stands, in case usage exhausts.
- **Usage failover (human directive, 2026-07-06):** if Claude usage limit reaches
  ~98%, offload the active job to Codex (`grind` profile for mechanical work,
  `review` for judgment-adjacent checks), park director-level judgment work in
  HANDOFF.md, and resume as director when the limit resets. Mark commits made in
  this mode `[codex-offload]`.

## Session continuity

`HANDOFF.md` outranks memory. At ~70% context or session end, execute the session-end
protocol (docs/LAUNCH_PROMPT.md section 9): land or park work, run the verification
loop, update HANDOFF.md (phase status, commits, open gates, degraded modes, next
three actions, live warnings), refresh the 60-second brief in docs/NEXT_SESSION.md,
append DECISIONS.md entries, commit everything, print a human summary with
one-line-answerable gate questions.

## Asana tracking

This project has an Asana board in the ATNI Climate Software portfolio.
At session close after a phase ship or notable progress, refresh it per
the ratified convention at `C:\dev\asana-sync\CONVENTION.md` (write split:
sessions post status updates, completion comments, task fields, and tasks
under ratified phases; milestones, due dates, deletions, and description
edits stay with the maintainer). Board gids: `C:\dev\asana-sync\ASANA_MAP.yaml`.
Never copy Asana gids or tracker metadata into this repository.

## Phase plan (acceptance criteria in docs/LAUNCH_PROMPT.md section 5)

0 Bootstrap | 1 Domain model + ADRs | 2 Rust core | 3 Frontend | 4 Ingestor |
5 Personal mode | 6 Hardening. Do not start N+1 until N passes or a DECISIONS.md
entry records the deferred criterion and why.

## Codex operating model

`docs/CODEX_GUIDE.md` is the manual. Codex CLI is the offload engine and sparring
partner via `codex exec` with pinned profiles `grind` and `review` (full model ids,
never aliases - alias drift burned the predecessor). If Codex is unavailable:
degraded mode - self-review, mark commits `[unreviewed-by-codex]`, log in HANDOFF.md,
do not stall.
