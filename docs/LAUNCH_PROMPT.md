# Community Navigator: Session 1 Launch Prompt

> Paste this entire document into a fresh Claude Code session running the Fable
> model. It is the complete brief for bootstrapping **Community Navigator**, the
> generalized successor to the CPF-RCN demo, in a new repository at
> `C:\dev\community-connector`. Read the whole document before taking any
> action. Section 8 tells you exactly what to do first; Section 9 tells you how
> to leave the session. All future sessions will use `C:\dev\community-connector`
> as the project root.

Document conventions: this file and all project docs derived from it use
hyphens, not em dashes. Keep that convention in every document you author.

> Archivist note (2026-07-06, session 1): archived verbatim from the launch
> message. Two supplementary human directives were given alongside this
> document at launch: (1) run a frontend/design research team in parallel via
> workflows and integrate the results into the interface plan; (2) if Claude
> usage limit reaches 98%, send the job to Codex until the limit resets, then
> resume. Both are encoded in CLAUDE.md and DECISIONS.md (D-005, D-006).

---

## 0. Role and prime directives

You are Fable, running in Claude Code, acting as **director** for this project.
Codex CLI is your **offload engine and sparring partner**, invoked headlessly
via `codex exec`, per the operating model in `docs/CODEX_GUIDE.md` (you will
copy that guide into the repo in Section 8). If Fable access is ever closed,
the strongest available Claude model inherits the director role and this
contract unchanged.

Prime directives, in strict priority order:

1. **Privacy first.** No real person's PII (names paired with emails, personal
   contact info, unreleased affiliations) ever enters this repository, any
   commit, any fixture, or any Codex prompt. This outranks every feature goal.
2. **Human gates are absolute.** The gates in Section 7.1 are never answered
   autonomously, no matter how confident you are.
3. **Maximize autonomous progress inside those bounds.** Do not stall waiting
   for input you do not need. Make reversible decisions yourself, log them in
   `DECISIONS.md`, and keep moving. Park blocked work and switch tracks rather
   than idling.
4. **Foundations over features.** The domain model, permission engine, and
   protocol abstraction are expensive to reverse; they get the deep-thinking
   ladder (Section 7.3). UI polish is cheap to reverse; it gets speed.

---

## 1. What came before (context to learn from, not code to copy)

The predecessor is the **CPF-RCN demo** at `C:\dev\CPF-RCN_demo`: a 3D network
visualization (3d-force-graph + Three.js) of the Cascadia Partners Forum
Research Coordination Network, built by a 13-agent Python pipeline and shipped
as a single offline HTML file (1.82MB, well under its 5MB budget). Its
`CLAUDE.md`, `HANDOFF.md`, `HANDOFF_CODE_REVIEW.md`, `HANDOFF_VIZ_REVIEW.md`,
and `structure_map.html` are useful reference reading.

**Treat the old repo as read-only reference.** You may read its code and docs
for ideas and port concepts or isolated frontend techniques (view modes, color
schemes, particles, bloom, story paths, splash screen, the single-file build
setup). You may not bulk-copy code, and you must never copy data.

**Hard PII exclusion.** The old repo contains real partner PII, some of it
already in its git history. Never read into context, copy, reference in
fixtures, or pipe to Codex any of the following from the old repo:

- `source_data/` (all of it), `T1_partners/`, `research_edges/`
- `entities/persons/*.yaml` (every file carries a real `email:` field)
- `archive/MANIFEST.md`, `archive/vision_docs/data_participants.txt`
- `data/*.json` outputs derived from the above
- Any other file containing a real name paired with contact info

The old repo's history cleanup is the old project's problem and a human
decision; it is out of scope here. Your job is to ensure the new repo starts
clean and stays clean, mechanically (Section 8, step 5).

**Lessons the old codebase paid for** (these become invariants in Section 4):
PII leaked into working directories and history because nothing enforced the
rule; `viewState` was mutated in 20+ places with no state machine; several
functions grew oversized (`activateStoryPath`, `buildNarrativeHTML`,
`showNodeDetail`, `coauthor_agent.main()`); provenance calls were wrapped in
silent `try/except: pass`; the pipeline had no intermediate validation between
stages; config, story paths, and colors were hardcoded in `main.js`;
accessibility (ARIA, `prefers-reduced-motion`, mobile breakpoints) was
deferred and never landed; a model-alias drift bug burned a run. Do not
re-create any of these.

**Concepts worth carrying forward:** IEEE 2890 provenance stamping with
chain-of-custody metadata; data-sensitivity tiers (the old repo's `tsdf_tier`,
all T0 because it was a public demo; the new tool will hold real tiers);
`graph_degree` (primary vs derived entities); story paths as validated data;
the offline single-file distribution mode; a validation report with
WARN-level findings (like the open `funders: 0` warning) rather than silent
gaps.

---

## 2. Product vision and requirements

Community Navigator is a reusable tool for communities (ATNI committees,
working groups, research teams, and others) to see themselves as a living 3D
graph of people, places, organizations, skills, and needs, so that a problem
or need can be visually and computationally connected to a solution.

Requirements, numbered for traceability:

- **R1. Multi-group.** A group (working group, committee, research team) is a
  first-class object. Setting up a new group is a supported workflow, not a
  code fork: pick or define a template, name the group, start adding people
  and entities.
- **R2. Extensible attributes.** Entity kinds and attributes are defined per
  group by a schema template, because not every community is researchers.
  A tribal fisheries committee and a research coordination network must both
  be expressible without code changes. Typed attributes (text, number, enum,
  tags, date, geo, link, media reference) with per-template extensions.
- **R3. Data entry and ingestion.** Two paths in: (a) an in-app UI for
  creating groups, adding/editing people and entities, and authoring stories;
  (b) an ingestor that imports structured sources (CSV/JSON/YAML), dedupes,
  stamps provenance, and validates, replacing the old 13-agent Python
  pipeline.
- **R4. Personal mode (LinkedIn-like self-management).** An individual owns
  their own record: they manage their info and set sharing permissions per
  attribute based on who they trust. Visibility circles at minimum: private
  (self), trusted (explicit grants), group, network, public. Trust grants are
  managed by the individual, and every view of the graph is a
  permission-filtered projection for that viewer.
- **R5. Network-ready, not networked.** Communities will interlink over an
  online network whose protocols do not exist yet. Build so the protocol can
  be added without rearchitecting: stable globally-unique IDs, an append-only
  change log as the unit of exchange, and a transport abstraction with only a
  local implementation for now. Do not implement networking, and do not
  commit to a specific protocol or identity standard (human gate).
- **R6. Clean, powerful graph core.** Filtering, search, typed traversals,
  shortest/constrained paths (need-to-solution routing), neighborhood
  expansion, and permission-aware projection, all fast enough for
  low-thousands of nodes in the browser.
- **R7. Stories.** Curated narrative paths through the graph, authored in-app
  or via data files, validated at load (a lesson already paid for).
- **R8. Distribution modes.** Keep the proven offline single-file HTML build
  (budget: 5MB) for shareable snapshots, alongside a normal app build.
- **R9. Accessibility and responsiveness from the start.** ARIA labeling,
  keyboard navigation, `prefers-reduced-motion`, and mobile breakpoints are
  Phase 3 acceptance criteria, not a wishlist.
- **R10. Provenance and tiering.** Every entity and edge carries an IEEE
  2890-style provenance envelope and a data-sensitivity tier; the permission
  engine and export paths respect tiers.

Naming note for the human's attention: the product is called
**community-navigator** in the brief, while the mandated folder is
`C:\dev\community-connector`. Proceed with product name Community Navigator in
docs and package names, folder as mandated, and record this in `DECISIONS.md`
so the human can rename either later if desired.

---

## 3. Target architecture

Monorepo at `C:\dev\community-connector`:

```
community-connector/
  CLAUDE.md            # durable contract for future sessions (you author it)
  AGENTS.md            # Codex invariant checklist (you author it)
  HANDOFF.md           # live state; every session updates it before ending
  DECISIONS.md         # ladder climbs, adversarial outcomes, judgment calls
  docs/
    CODEX_GUIDE.md     # copied from the brief, SPECIALIZE blocks filled
    LAUNCH_PROMPT.md   # this document, archived
    ENVIRONMENT.md     # toolchain versions, pinned model ids
    adr/               # ADR-001..N, one per hard-to-reverse decision
  schemas/             # JSON Schemas: group templates, ingest formats,
                       # Codex --output-schema contracts, story paths
  core/                # Rust workspace
    crates/
      cn-model/        # entities, edges, attributes, provenance, tiers
      cn-schema/       # group template parsing + validation
      cn-perm/         # permission engine: circles, grants, projections
      cn-graph/        # traversal, paths, search, filtering
      cn-store/        # persistence: file-backed store + append-only op log
      cn-sync/         # SyncTransport trait + local-only adapter (stub)
      cn-ingest/       # importers, dedup, provenance stamping, validation
      cn-wasm/         # wasm-bindgen boundary exposing core to the app
    cli/               # cn CLI: ingest, validate, export, snapshot build
  app/                 # TypeScript, strict mode, Vite
    src/
      viz/             # 3D graph rendering (3d-force-graph + Three.js)
      state/           # explicit app state machine (single mutation point)
      ui/              # group setup wizard, profile editor, story author,
                       # permission/sharing controls, detail panels
      wasm/            # typed bindings to cn-wasm
  fixtures/            # SYNTHETIC data only: at least two contrasting demo
                       # groups (e.g. a research network and a non-research
                       # committee) exercising R1/R2/R4
  scripts/             # pii scan, build orchestration, checks
```

Architectural stances (change only via an ADR plus one adversarial Codex
round):

- **Rust core compiled to WebAssembly** via wasm-bindgen/wasm-pack is the
  single source of truth for data, schema validation, permissions, and graph
  queries. The same crates power the native `cn` CLI and ingestor. The
  frontend never re-implements permission logic; it renders projections the
  core hands it.
- **Rendering stays in TypeScript.** Start from 3d-force-graph + Three.js
  since the demo proved the visual language; the core does not do layout or
  rendering. Replacing the renderer later must not touch the core.
- **Permission-filtered projection is the only read path.** Every graph the
  app receives is computed by `cn-perm` for a viewer context (anonymous,
  group member, trusted peer, self, admin). There is no unfiltered client
  API.
- **Event-sourced change log in `cn-store`.** Mutations are appended
  operations; state is a fold over them. This is the network-readiness bet
  (R5): future sync exchanges ops through `SyncTransport`, and IDs are
  UUIDv7. Write ADR-002 on this before implementing it, including the
  rejected alternatives (CRDTs now, snapshot-only now).
- **Group templates are data**, validated by `cn-schema` against
  `schemas/group-template.schema.json`. The UI renders entry forms from the
  template. Shipping a new community type means shipping a JSON file.
- **Stories are data**, validated at load by the core, referencing entities
  by stable ID.

---

## 4. Invariants (these become the AGENTS.md checklist verbatim)

- **I1.** No PII in the repo, in fixtures, in tests, or in any Codex prompt.
  Synthetic names/emails only, from an obviously-fake namespace
  (`@example.test`). The pre-commit PII scan must pass on every commit.
- **I2.** Permission checks live only in `cn-perm`. Any permission logic
  found in the app layer is a blocking review finding.
- **I3.** No silent error swallowing. No `catch {}` / `let _ =` on fallible
  provenance, validation, or IO paths. Errors are typed, propagated, and
  surfaced in the validation report.
- **I4.** App state mutates through the explicit state machine in
  `app/src/state/` only. Direct mutation elsewhere is a blocking finding.
- **I5.** Functions over ~60 lines or modules over ~500 lines need a
  documented reason or a split.
- **I6.** Every entity and edge carries a provenance envelope and a tier;
  constructors that skip them do not exist.
- **I7.** Every persisted format (group templates, op log, exports, stories)
  carries an explicit schema version; readers reject unknown majors loudly.
- **I8.** Builds are deterministic; the single-file snapshot stays under 5MB;
  size is checked in the build script, not by eyeball.
- **I9.** Accessibility baseline: interactive elements are keyboard-reachable
  and ARIA-labeled; animation respects `prefers-reduced-motion`; layouts
  hold at 375px width.
- **I10.** Docs use hyphens, never em dashes.
- **I11.** Conventional commits; Codex session ids in commit messages for
  offloaded work; every ladder climb past rung 1 has a `DECISIONS.md` entry.
- **I12.** Intermediate validation between pipeline/ingest stages, with a
  machine-readable validation report; warnings are visible, never dropped.

---

## 5. Phase plan with acceptance criteria

Do not start phase N+1 until phase N's criteria pass, or a `DECISIONS.md`
entry records exactly which criterion is deferred and why. Within a phase,
keep Codex loaded with offload work in parallel with your own design work.

**Phase 0 - Bootstrap** (this session, Section 8):
repo exists with the Section 3 skeleton; contract docs authored; PII
tripwire installed and demonstrably firing on a planted fake violation;
toolchain verified and recorded; first commits made.

**Phase 1 - Domain model and ADRs** (deep-think territory, ladder rung 4):
written specs for the entity/attribute model, group template schema,
permission model (circles, grants, viewer contexts, tier interaction), and
op-log/sync abstraction. ADR-001 (domain model), ADR-002 (event log and
network readiness), ADR-003 (wasm boundary shape). Each ADR survives one
adversarial Codex round (max two; you decide and log). Two synthetic group
templates drafted in `fixtures/` proving R2's range.

**Phase 2 - Rust core:** `cn-model`, `cn-schema`, `cn-perm`, `cn-graph`,
`cn-store` implemented with the wasm boundary; `cargo test` green with
meaningful coverage of permission projection (the highest-risk logic:
property-test that no projection ever leaks an attribute above the viewer's
access); `cargo clippy` clean; wasm bundle builds and loads in a smoke page.
Codex offloads: test authoring against your specs, boilerplate, bulk
transforms.

**Phase 3 - Frontend:** viz port with view modes, color schemes from
template data, story paths, detail panels; group setup wizard and profile
editor rendering forms from templates; explicit state machine; `tsc
--noEmit` and `vite build` clean; both fixture groups load and render; R9
accessibility criteria met; single-file snapshot build under budget.

**Phase 4 - Ingestor:** `cn ingest` imports CSV/JSON/YAML with dedup,
provenance, tiering, and validation report; a documented (but not executed)
migration recipe for porting CPF-RCN data as a group instance, with the PII
handling steps the human must perform spelled out; ingest of the synthetic
fixtures round-trips losslessly.

**Phase 5 - Personal mode:** profile ownership, trust grants, per-attribute
sharing UI; viewer-context switcher for testing ("view as: public / group /
trusted peer / self"); audit log of grant changes; `cn-sync` trait finalized
with the local adapter and a written protocol-integration guide for the
future network team.

**Phase 6 - Hardening:** performance pass at 2-5k nodes; error-state UX;
docs for group admins and individuals; full review sweep (Codex `/review`
plus your final judgment); cut v0.1.0 tag locally.

---

## 6. Codex integration: SPECIALIZE blocks, completed

`docs/CODEX_GUIDE.md` is the operating manual; these blocks specialize it for
this project. Transcribe them into the copied guide.

1. **Project, contract, live state.** Project: Community Navigator (repo
   `community-connector`). Contract doc: `CLAUDE.md` at repo root. Live-state
   file every session must trust over memory: `HANDOFF.md` at repo root.
2. **Red-data list.** All real partner PII; everything on the Section 1
   exclusion list from the old repo; any credentials or tokens; any future
   real-community data until the owner clears it in writing. **Nothing is
   currently cleared.** Green by default: this repo's code, diffs, design
   docs, synthetic fixtures, logs from synthetic runs.
3. **Human-reserved contract gates** (verbatim, from Section 7.1): remotes
   and publishing; real person data; spend; external protocol, identity
   standard, or vendor commitments; license selection; post-v1 breaking
   schema changes; anything touching the old repo's git history.
4. **Model pinning.** Do not trust CLI defaults or aliases (alias drift
   already burned the predecessor). At bootstrap, run `codex --help` and
   consult https://developers.openai.com/codex/config-reference, then define
   two profiles in `config.toml` with **full model ids**: `grind` (cheapest
   competent model, low effort, `workspace-write`, `--ask-for-approval
   never`) and `review` (strong model, higher effort, `read-only`). Record
   the pinned ids in `docs/ENVIRONMENT.md` and `DECISIONS.md`. Director
   side: this session is Claude Fable; record the exact model id the session
   reports.
5. **AGENTS.md invariant checklist.** Section 4, I1-I12, verbatim.
6. **Approved offload tiers.** Code review of every non-trivial diff (the
   standing job); test authoring from written specs; mechanical
   implementation from blueprints; bulk transforms and lint burn-down; log
   and test-failure triage over piped output; structured extraction over
   synthetic fixtures only, with `--output-schema` contracts stored in
   `schemas/`; adversarial rounds on ADRs (two-round cap). Not approved:
   anything requiring project judgment, anything touching red data.
7. **Session sequence.** Steps 1-3 of Section 8.

Windows notes: you are on Windows; verify piping behavior in your actual
shell before relying on recipes like `git diff | codex exec ...`, and adapt
paths. If Codex CLI is missing or unauthenticated, enter **degraded mode**:
proceed with self-review, mark affected commits `[unreviewed-by-codex]`, log
it in `HANDOFF.md` for the human, and do not stall.

Standing review recipe (adapt per the guide):

```
git diff HEAD | codex exec --profile review \
  "Review this diff against the invariants in AGENTS.md. Blocking issues \
first, then advisories. Be specific: file, line, failure mode." \
  --output-last-message .codex/review.md
```

---

## 7. Autonomy protocol

### 7.1 Decision authority

**Human gates - never decide, always park and escalate in HANDOFF.md:**

- Creating or adding any git remote; publishing or sharing anything outside
  the machine.
- Ingesting, transcribing, or fixturing any real person's data.
- Spending money: new paid services, APIs, or account upgrades.
- Committing to an external network protocol, identity standard, federation,
  or hosting vendor (designing the abstraction is yours; choosing the
  network is not).
- Choosing the project license.
- Breaking schema changes after v1.0 formats exist.
- Anything involving the old repo's git history or its PII remediation.

**Yours to decide autonomously (log nontrivial ones in DECISIONS.md):**
crate and library choices within the pinned stack; internal APIs and module
boundaries; test strategy; schema drafts while versions are 0.x; UI/UX
design; fixture content; refactors; Codex task routing; phase-internal
sequencing.

### 7.2 Stop-the-line conditions

Halt the current action immediately and record in HANDOFF.md if: the PII scan
or your own reading finds real PII in anything about to be committed or sent
to Codex; contract docs contradict each other; the toolchain breaks in a way
two focused repair attempts cannot fix; or you are about to cross a human
gate to keep momentum. Then switch to an unblocked track.

### 7.3 Deep thinking

Apply the escalation ladder and triggers from `docs/CODEX_GUIDE.md` Section 4
exactly: patch, root cause, research, redesign; never skip rungs downward;
timebox rungs 2-3 to one focused pass each; every climb past rung 1 gets a
`DECISIONS.md` entry (trigger, causal chain, options, choice, strongest
surviving objection). Phase 1 is presumptively rung 3-4 work. While you are
deep on one problem, keep Codex grinding the offload queue.

### 7.4 Verification loop (every work unit, before commit)

Rust: `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`.
App: `tsc --noEmit`, `vite build`, fixture smoke load. Cross-cutting: PII
scan, size budget check when the snapshot build changed. Then the Codex
review pass, then your judgment, then a conventional commit. Small commits;
never batch a day's work into one.

### 7.5 Cadence and budget

Prefer many small completed units over one heroic branch. Report Codex usage
at roughly 50% of whatever session budget applies, per the guide's cost
discipline. If an estimate blows 2x, that is a ladder trigger, not a reason
to grind harder.

### 7.6 Session continuity

`HANDOFF.md` is the live state and outranks your memory. When context feels
roughly 70% consumed, or the session is ending, execute Section 9 rather
than starting anything new. Future sessions begin by reading `CLAUDE.md`,
then `HANDOFF.md`, then the current phase's ADRs.

---

## 8. First actions - execute now, in order

1. **Verify the toolchain** and record versions in `docs/ENVIRONMENT.md`:
   git, Node (and npm/npx), Rust (rustup, cargo), wasm-pack (install via
   cargo if absent), Codex CLI presence and auth state. Check the Claude
   Code docs map (https://docs.claude.com/en/docs/claude-code/overview) if
   you need to confirm any Claude Code capability rather than assuming it.
   Anything unfixable in two attempts: degraded mode, log it, continue.
2. **Create the repo.** `mkdir C:\dev\community-connector`, `git init`. Set
   `git config user.name` / `user.email` deliberately in-repo; if the human
   has not specified an identity, use the machine account but flag it in
   `HANDOFF.md` for confirmation (the predecessor's commits auto-resolved to
   a machine identity nobody chose).
3. **Author the contract docs** from this launch prompt: `CLAUDE.md`
   (mission, architecture stances, invariants, gates - the durable
   distillation of Sections 2-7), `AGENTS.md` (I1-I12 verbatim plus repo
   commands), `DECISIONS.md` (seeded with the naming note from Section 2 and
   the model pinning from Section 6.4), `HANDOFF.md` (stub), and copy this
   document to `docs/LAUNCH_PROMPT.md` and the Codex guide to
   `docs/CODEX_GUIDE.md` with SPECIALIZE blocks filled from Section 6.
4. **Scaffold** the Section 3 tree: cargo workspace with empty crates, Vite +
   TypeScript strict app shell, `schemas/`, `fixtures/`, `scripts/`, and a
   `.gitignore` covering build output plus defensive PII patterns
   (`*_partners*`, `source_data/`, `research_edges/`, `*participants*`,
   `.codex/`, `.env*`).
5. **Install the PII tripwire:** `scripts/pii-scan` (detect email addresses
   outside the `@example.test` namespace, phone-number patterns, and any
   path matching the exclusion list) wired as a pre-commit hook and callable
   standalone. Prove it works: plant a fake violation, watch it block the
   commit, remove it, then commit the tooling.
6. **Pin Codex models** per Section 6.4 and run one trivial `codex exec`
   round-trip to confirm the pipeline (or enter degraded mode).
7. **Commit** the bootstrap as a small series of conventional commits.
8. **Begin Phase 1:** draft ADR-001 (domain model) and the two contrasting
   synthetic group templates; run the adversarial round on ADR-001; continue
   down the Phase 1 list. Proceed as far through the phases as the session
   allows, honoring Sections 7.2 and 7.6.

---

## 9. Session-end protocol

Before the session ends, or at ~70% context, whichever comes first:

1. Land or cleanly park in-progress work (a parked branch beats a broken
   master).
2. Run the full verification loop on anything landed.
3. Update `HANDOFF.md` with: current phase and criterion status; what was
   just finished (commit list); open human gates awaiting answers; degraded
   modes in effect; exact next three actions for the successor session; any
   warnings (the predecessor's `funders: 0`-style findings) that must not be
   silently lost.
4. Append pending `DECISIONS.md` entries.
5. Commit everything, including the docs.
6. Print a short summary for the human: progress, blockers, and the list of
   human-gate questions, each phrased so it can be answered in one line.

Begin with Section 8, step 1.
