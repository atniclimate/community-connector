# HANDOFF.md - Live State (pickup target)

> The /pickup target; outranks session memory. Reading order for a fresh session:
> `CLAUDE.md`, this file, `docs/NEXT_SESSION.md` (launch cards), then
> `docs/planning/CONVENTION-SPRINT-2026-09-14.md` (the sprint plan of record) and
> `docs/planning/NEXT-PHASE-convention-e2e.md` for what comes after the convention.
> Written 2026-09-14 at the end of the convention sprint. Previous handoff:
> `docs/archive/handoffs/2026-09-13-trueup.md`.
>
> **The repo is PUBLIC** (origin `https://github.com/atniclimate/community-connector`).
> Plain `git push` fails from a non-interactive session here: `gh` has two accounts and the
> active one is not `atniclimate`. Working one-shot form (changes no config, prints no token):
> `git -c credential.helper= -c 'credential.helper=!f() { echo username=atniclimate; echo "password=$(gh auth token --user atniclimate)"; }; f' push origin main`.

## What this project is

Community Navigator (repo folder `community-connector`) is a local-first, privacy-first
tool for a community to see itself as a permission-filtered 3D graph of people,
committees, organizations, skills, and needs, and to route a need to people who can meet
it. A Rust core compiled to WASM owns data, schema, permissions, and graph queries; the
TypeScript app renders only the viewer-scoped projections it is handed. The immediate
deliverable is the A9 constellation that closes the ATNI General Assembly session on
**Tuesday 2026-09-15 at about 11:45**: after a 30-minute role-play focused on one Nation,
the close reveals that Nation as one node among Relatives at ATNI, that shared priorities
connect them, and that the interconnection surfaces the Tribes, allies, and affinities
that make complex tasks achievable. It runs on synthetic data on the presenter's laptop.

## Where everything lives

| What | Path |
|---|---|
| Durable contract (mission, R1-R10, gates, autonomy) | `CLAUDE.md` |
| Invariants I1-I12 (review standard) | `AGENTS.md` |
| Decision register (D-001..D-103 with addenda) | `DECISIONS.md` |
| Convention sprint plan of record (beats, waves, parked gates) | `docs/planning/CONVENTION-SPRINT-2026-09-14.md` |
| Post-convention plan (S-E1..S-E5, wow slice, go-live checklist) | `docs/planning/NEXT-PHASE-convention-e2e.md` |
| Session plan of record (R-sessions) | `SESSION_ROSTER.yaml` |
| Route map to 1.0 and task trace | `PLAN_1.0.md`, `TRACE.yaml` |
| Reveal launcher, key table, and the A9 cue sheet | `scripts/reveal.ps1`, `CONTROLS.md` |
| Presenter beats (eight, the last is the finale) | `app/public/beats.atni.json` |
| ATNI convention synthetic template, fixture, generator, edge checker | `fixtures/templates/atni-convention.template.json`, `fixtures/groups/atni-convention.ops.jsonl`, `app/scripts/generate-atni-ops.mjs`, `app/scripts/check-atni-edges.mjs` |
| ATNI design system digest (source stays off-repo at `I:\ATNI_design-system\`) | `docs/design/atni-design-system-digest.md` |
| Convention spine (off-repo) and this sprint's reconciliation note beside it | `I:\ATNI-annual-convention-2026\presentation-spine-extracted\community-connector-reconciliation-2026-09-14.md` |
| Verification battery (12 members) and pre-commit hook | `scripts/check-all.ps1`, `scripts/hooks/pre-commit` |
| Rust core and `cn` CLI | `core/crates/cn-{model,schema,store,perm,graph,api,wasm,ingest,sync}`, `core/cli` |
| Remote intake (built, accepted D-089, not deployed) | `form/`, `relay/`, `core/cli/src/intake/` |
| App (renderer `viz/`, presenter `viz/presenter.ts`) | `app/src/{viz,ui,state,wasm,theme}` |
| S-E2 intake-to-edges, BRANCH ONLY, verdict owed by the human | branch `s-e2-intake-edges`, worktree `I:\claude-temp\claude\I--community-connector\057c2e60-c176-425e-bd6e-5655e5ac0ff2\scratchpad\wt-s-e2` |
| Codex review lane (off-repo) | `C:\dev\_reviews\community-connector\` (latest: `2026-09-14_convention-sprint-review.md`, `-b2.md`, `-b3.md`) |
| Browser-walk screenshots (headless, synthetic) | `I:\claude-temp\claude\I--community-connector\057c2e60-c176-425e-bd6e-5655e5ac0ff2\scratchpad\walk*\` |

## State of play

**DONE (verified at code HEAD `202e762`; last full `pwsh scripts/check-all.ps1` = 12/12
PASS at `202e762`; only docs commits since):**
- Convention sprint 2026-09-14 (D-103 and addenda), executed by a 22-agent discovery
  workflow and a 15-agent execution workflow plus follow-ups:
  - S-R0 finished: `rust-toolchain.toml` pins 1.98.1; `troika-three-text` is a runtime
    dependency.
  - Fixture: person-to-person `connected_to` edges derive from shared
    `areas_of_interest` tags (60 edges, 48 of 60 people tied); a documented spotlight
    person (`00000000-0000-0000-0000-0000000de2c5`, fictional tribe) sits in Energy and
    Climate Resilience with 6 priority-sharing neighbors; `npm run check:fixture` guards it
    inside the `app-templates` check-all member.
  - Presenter: `camera` (`hold` / `fit` / `fly`), `filter.edgeKinds`, `labelKinds`
    (fail-closed stage name gate: no person label or hover name ever renders in present
    mode), count-only measure captions, validated beat sheet, hotkeys `c o m p 1 End`, an
    operator rail hidden by default and toggled with `r` (state in `presentation.railShown`).
  - Eight beats authored; the A9 cue sheet is in `CONTROLS.md` (three Space presses from
    the spine's single documented cue; alternative pre-switch offered to the human).
  - ATNI stage styling slice: presenter caption in League Spartan SemiBold 600
    (self-hosted `@fontsource/league-spartan` 5.2.8, zero runtime font requests), ground
    `#010B13`, text `#E8ECF0`, kind colors untouched.
  - Intake: live in-app entry rehearsed on the ATNI template up to the native folder
    picker; `cn intake selftest --dry` passes; a silent pre-grant error was found and fixed.
  - QR-target form (D-104, morning of 2026-09-14): `form/` now builds against the ATNI
    template with `-Kinds person` (`pwsh scripts/build-form.ps1 -TemplatePath
    fixtures/templates/atni-convention.template.json -Kinds person`), styled to the ATNI
    design system (Black BG ground, League Spartan title, Arial/Arimo body, ATNI Red
    button, Under Review draft tag, square bullets, 640px column) and worded in the public
    voice (labels Name / Tribal Nation or organization / Role / Priority areas / ...);
    consent draft v2 at `docs/design/intake-consent-text-draft-2026-09-14.md` is shared
    by the form and the in-app path; required-field errors show only after interaction.
    The form asks the human's nine questions verbatim (D-105,
    `docs/design/intake-questions-2026-09-14.md`); the template is 0.1.2 with the new
    tags attributes `roles`, `origins`, `connections`, `seeking`, `offering`,
    `committee_memberships`, `organization_affiliations` (additive; fixture unchanged).
    Still DRAFT (D-023), still not deployed (D-059.8). Preview locally with
    `cd form && npx vite preview --port 4173` at `/community-connector/`.
    **Degraded mode:** Codex hit its usage limit (retry after 2026-09-19), so the form
    commits `256f47f..` through the contrast fix are `[unreviewed-by-codex]`; a
    documented self-review with an independent contrast check stands in
    (`C:\dev\_reviews\community-connector\2026-09-14_form-atni-review.md`, D-104).
  - Verification: check-all 12/12, app suite green, PII scan clean, a real-Chromium walk of
    every beat, key, and rail button under normal and reduced-motion settings (all
    assertions passing), three Codex review rounds with every blocking finding fixed and
    every advisory dispositioned (D-103 addenda 1 and 2).
- Everything before the sprint (relay steps 1-11, reveal chain, viz quick wins, D-097 to
  D-102): unchanged; see the archived handoffs.

**NOT done - ordered next actions:**
1. **Human real-GPU walkthrough on the presentation laptop and projector** (S-R4b gate,
   the one check no agent can do): `pwsh scripts/reveal.ps1`, click Present, `Home`, then
   Space through all eight beats, hold the last one 60 seconds. Then viewport-only
   screenshots, eyeballed, committed to `docs/design/screenshots/`.
2. **Answer the parked one-liners** in `docs/planning/CONVENTION-SPRINT-2026-09-14.md`
   section 4 (OQ-10 confirm; Makah naming; camera hold vs fly on the one-node beat; cue
   timing; consent line stays spoken; QR lines; operator; S-E2 verdict).
3. **Apply the spine reconciliation note** (or not) in the convention folder; the note is a
   proposal beside the YAML, nothing was edited there.
4. **S-E2 verdict** (D-056.1/2): read `docs/blueprints/intake-edges.md` on the branch and
   the three reviewer lenses in D-103 addendum 2; record the verdict in DECISIONS.md;
   merge after the convention (expect one import-line conflict in
   `app/src/ui/intake/panel.ts`, resolution is the union; regenerate the fixture so the
   embedded template carries 0.1.2).
5. Post-convention: S-E1 remainder (section 7 answers), S-E4, S-E5 / S-R1 (gated), the
   deferred advisories (beat-sheet `schema_version`, `index.ts` split, wiring-level tests),
   Codex re-review of `0cb27f0..d0e15a0`, W2/W4/W5.
- Known unverified: `relay/` and `form/` vitest suites have not been re-run since
  2026-08-11; they are not check-all members. Headless pixels are dark; the night sky has
  not been seen on a real GPU since the sprint.

## The human's queue

1. **Real-GPU and projector walkthrough today** (above). Every viz receipt is headless.
2. **The eight parked one-liners** (sprint plan section 4). Defaults if unanswered:
   pregenerated synthetic; fictional tribe; camera hold; three presses from the single cue;
   consent line spoken; QR lines are the spine owner's call; operator unnamed; S-E2 parked.
3. **Brand reconciliation** (digest section 7) still open: which design-system edition is
   the sole authority (the 07/16 bundle in `I:\ATNI design system.zip` or the 09/12
   revision in `I:\ATNI_design-system\`); the D-104 typography mapping holds under either.
   Also: D-023 review of consent draft v2 (`docs/design/intake-consent-text-draft-2026-09-14.md`),
   and the public product name on the form ("Community Connector" vs "Community Navigator").
4. **Go live at the convention or not** (D-090c): unchanged, every deploy-bar and real-data
   row is still NOT DONE; the sprint assumed no.
5. **Record, amend, or reject D-090c..D-096c**; D-096c (toolchain pin) is now done.
6. Standing: G-BACKUP accepted not solved; pilot-window close needs the recorded
   rejected-record purge sweep (D-059.11).

## Non-negotiables a fresh session must not violate

- **Public repo, privacy first (I1):** no real-person PII in any file, commit, fixture, or
  Codex prompt; never commit `_private/`. Never copy ATNI logos, font files, or the
  design-system source into the repo; fonts ship via pinned open-licensed packages
  (D-101). On the projector, counts, never names (D-099); the render path now enforces it
  in present mode.
- **Deploy bar D-059.8 is UNMET:** nothing goes live on Pages or Workers. No real ingestion
  before the recorded committee checkpoint. Cloudflare spend only for the relay, tier is a
  human call.
- **The graph never listens (ADR-005):** new entries reach the app only by a facilitator
  action (apply, then reload).
- Permission logic only in `cn-perm` (I2); app state mutates only through `app/src/state`
  (I4); provenance + tier on everything (I6); versioned formats (I7); snapshot under 5MB
  (I8); hyphens in docs (I10); reduced-motion variant for every motion (I9).
- ADR-004 renderer stands. Standing rulings no session may "fix": D-032, D-037, D-051,
  D-056.4. The S-E2 branch is never merged without the human's recorded verdict.
- Verification loop before every commit; atomic conventional commits; stage only your own
  paths and pass an explicit pathspec to `git commit` when another session shares the
  checkout (a concurrent commit swept staged files once on 2026-09-14 and was corrected).

## Key design commitments (shortest refresher)

- Rust/WASM core is the single source of truth; the app renders permission-filtered
  projections only; the op log is event-sourced and state is a fold over it.
- Intake: in-app facilitator entry is primary; the remote QR path is built and accepted
  but not deployed; `cn intake apply` is the only writer.
- Presenter mode: beats from `app/public/beats.atni.json`, validated at load; kind and
  edge-kind beats dim non-matching entities in place; measure beats use `graph_measures`;
  `labelKinds` gates labels; `camera` controls motion; Escape exits; `r` toggles the rail.
- TSDF tier codes primary in the UI (D-032); in-app story authoring in v0.1 (D-037).
