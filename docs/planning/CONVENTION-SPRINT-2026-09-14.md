# Convention sprint 2026-09-14: the A9 constellation

Status: EXECUTING. Written 2026-09-14 00:50 by the orchestrating session after a
22-agent discovery-and-sort workflow (13 read-only scouts over this repo, the convention
spine at `I:\ATNI-annual-convention-2026`, and the ATNI design system at
`I:\ATNI_design-system`; 5 sorters with intake / analysis / display / narrative /
gates lenses; one synthesis; 3 skeptics). Decision record: D-103. Live state authority
remains `HANDOFF.md`.

Deadline: Tuesday 2026-09-15, General Assembly slot at about 11:45. The constellation
is act A9, Patrick's close. It resolves on "communities connected in new ways" and holds
through applause. Nothing follows it, and the run of show says it is never cut.

## 1. What the room must see

The 30-minute role-play focuses tightly on one Nation and one inquiry (clean energy,
resilience, climate risk, funding, land use). The close reverses the lens: that Nation
is one node in a large community of Relatives at ATNI. Every node carries its own great
challenges. Where people work on the same goals, the interconnection advances all of
them and surfaces the Tribes, allies, and affinities that make complex tasks achievable.

Beat sequence (presenter mode, `app/public/beats.atni.json`, advanced with Space):

| # | Beat id | On screen | Carries |
|---|---|---|---|
| 1 | `network-overview` | whole projection fit to frame | pre-show, rehearsal |
| 2 | `members` | people foregrounded | floor conversation |
| 3 | `committees` | committee rings foregrounded | floor conversation |
| 4 | `organizations` | partner organizations foregrounded | floor conversation |
| 5 | `connectors` | top-N betweenness people lit, count-only caption | floor conversation ("the people who connect committees") |
| 6 | `one-node` | ONE synthetic person lit by opacity among ~86 dim nodes; camera holds the full frame; caption "One Nation. One node." | "communities connected in new ways" |
| 7 | `shared-priorities` | that node's priority-sharing Relatives and the shared-priority edges light; caption gives the count | "But the ways that matter most are as relatives." |
| 8 | `constellation` | everything lit, fit to frame, caption with the three counts; holds indefinitely | "Relationships. Being a good relative. Being a good ancestor." Hold. |

Captions show counts, never names (D-099). No tool names are spoken in A9. The consent
line "Displayed at the level you agreed to. Exists for this room." stays SPOKEN, never a
screen, per the spine's recorded decision (open-questions.md, decisions already made).

## 2. Shape of Tuesday (orchestrator's recommendation, pending the human's OQ-10 answer)

Pregenerated, fully synthetic constellation from `fixtures/groups/atni-convention.ops.jsonl`
(60 people, 15 committees, 12 organizations). No deploy, no keygen, no relay, no real
data. Live in-app facilitator entry is rehearsed as an upgrade against a scratch copy and
can never touch the committed fixture. Every deploy-bar and real-data gate stays parked.

Why: the deploy bar D-059.8 is unmet, the ATNI Climate committee approval is not
recorded, and no registration-export ingest tool exists. The synthetic fixture already
carries every edge the audience will see. This is the spine's own OQ-10 recommendation.

## 3. Critical path (ids CS-xx; executor in brackets)

Wave A, parallel, disjoint files:

- **CS-01 [sonnet] S-R0 remainder.** `rust-toolchain.toml` pinned to the installed
  toolchain (`rustc --version`, 1.98.1 today; `docs/ENVIRONMENT.md` row updated),
  `troika-three-text` moved from devDependencies to dependencies in `app/package.json`.
  Accept: `npm ci && npm run build` in `app/` succeeds; cargo builds under the pin.
- **CS-02 [sonnet] Shared-priority edges in the fixture generator.** In
  `app/scripts/generate-atni-ops.mjs`, person-to-person `connected_to` edges are derived
  deterministically from overlapping `areas_of_interest` tags (two or more shared tags;
  cap the total so the fixture stays legible), replacing the arbitrary `(i*13+7)`
  formula. One designated spotlight person (a stable, documented UUID) carries an
  energy-plus-resilience priority mix so the `one-node` and `shared-priorities` beats read
  true. Tribe names stay fictional (Makah naming parked, OQ-02). Fixture regenerated and
  committed; all Rust and app tests that read the fixture still pass. Accept: every
  `connected_to` edge's endpoints share at least two `areas_of_interest` tags (checked by
  a script); `pwsh scripts/pii-scan.ps1` clean.
- **CS-03 [sonnet] Presenter beat capabilities.** In `app/src/state/state.ts`,
  `app/src/viz/presenter.ts`, `app/src/viz/index.ts` (and `edges.ts` / `focus.ts` only if
  needed): (a) optional `camera: "hold" | "fit" | "fly"` on `PresentBeat` (default keeps
  today's behavior; `hold` keeps the current camera, so an opacity-only spotlight
  satisfies the stage's "outputs resolve in, no fly-ins" rule); (b) optional
  `filter.edgeKinds` that lights the entities incident to those edge kinds and keeps
  those edges bright while others dim; (c) `focusEntityId` beats highlight the node and
  its neighbors by opacity without requiring a camera flight; (d) measure-beat captions
  become count-only ("label - N highlighted"), explanations no longer concatenated on
  stage; (e) a vitest that loads `beats.atni.json` and the fixture and asserts every
  `focusEntityId` resolves to an EntityCreate id and every `measure` is a
  `PresentMeasure`. ADR-004 stands: no bloom, particles, new render pass, or new runtime
  dependency. Accept: `npm run typecheck && npm run build && npm run test` green with
  new tests for each of (a) to (e).

Wave B, after A, parallel, disjoint files:

- **CS-04 [sonnet] Author the beats and the A9 cue sheet.** `beats.atni.json` gains
  `connectors`, `one-node`, `shared-priorities`, `constellation` (last element, no filter,
  no measure, no focus). `CONTROLS.md` gains "A9 cue sheet (2026-09-15)": the display
  is pre-loaded on beat 5 during the reports; at the single documented cue
  "communities connected in new ways" the operator presses Space to `one-node`, then
  Space at "as relatives", then Space at "Relationships"; nothing after; Escape never;
  recovery is Home then Space seven times; an alternative pre-switch at "These tools can
  be more" is listed as an option for the human; "Operator: ________" left blank
  (OQ-11). Also documents the Wave B keys below.
- **CS-05 [sonnet] ATNI stage styling (S-E1 slice that needs no section-7 answer).**
  Presenter caption in League Spartan SemiBold 600 (D-101) via a pinned open-licensed
  `@fontsource` package, self-hosted, no runtime font requests; stage ground `#010B13`
  and caption text `#E8ECF0` from the design-system digest applied in present mode
  through the existing theme-token pipeline; ATNI Red `#E13D33` reserved for the
  selection ring only if contrast passes the digest's rule. Nothing else restyled.
- **CS-06 [sonnet] Presenter keys and rail (S-E3 slice).** In present mode `c`, `o`,
  `m`, `p`, `1`, `End` jump to `committees`, `organizations`, `members`,
  `shared-priorities`, `one-node`, `constellation` by beat id; a small ARIA-labeled
  button rail mirrors them, visible only in present mode, keyboard reachable, holds at
  375px, no new animation. Unit tests for the key dispatch. Does not edit `CONTROLS.md`
  (CS-04 owns it).

Wave C, serial verification:

- **CS-07 [sonnet] Full loop.** `pwsh scripts/check-all.ps1` 12/12, `pii-scan`
  self-test and scan, `git status` clean, HEAD recorded.
- **CS-08 [sonnet] Browser walk.** Real Chromium against `scripts/reveal.ps1`: Present,
  Space through all eight beats asserting caption text, zero console errors, the
  one-node beat changes the highlight set, the shared-priorities beat lights more than
  one node, the constellation beat holds 60 seconds; a screenshot per beat to the
  scratchpad (headless pixels may be dark; that is the human's check).
- **CS-09 [codex-review] Codex review** of `f9c764d..HEAD` with the pinned review
  profile; blocking findings fixed and CS-07 re-run, or dispositioned in DECISIONS.md.
  If Codex is unavailable, self-review and mark `[unreviewed-by-codex]`.

Wave D, optional upgrades, after C is green:

- **CS-10 [sonnet + human] Live intake rehearsal** on the ATNI template against a
  scratch copy per `docs/runbooks/live-entry.md`; the directory-picker step is handed to
  the human. Known accepted gap: a person added live lands without edges until S-E2.
- **CS-11 [sonnet, branch only] S-E2 intake-to-edges** built on branch `s-e2-intake-edges`
  with the mandatory three-reviewer adversarial round; NOT merged; verdict recorded by the
  human (D-056.1/2).

Wave E, close:

- **CS-12 [orchestrator]** HANDOFF.md, `docs/NEXT_SESSION.md`, DECISIONS.md
  dispositions; convention-side reconciliation note drafted as a new file beside the
  spine (never editing the spine's YAML); push; Asana status.

## 4. Parked for the human (one line each answers it)

1. OQ-10: confirm Tuesday is pregenerated synthetic only (recommended: yes).
2. Name Makah on the spotlight node, or keep it fictional until OQ-02 clears (default: fictional).
3. One-node beat: opacity spotlight with the camera holding (default, complies with "no fly-ins"), or a camera fly-in (set `"camera": "fly"` in the beat).
4. Cue timing: three Space presses from the single documented cue (default), or a pre-switch at "These tools can be more" (needs a spine edit).
5. Consent line stays spoken, never a screen (recorded decision): confirm.
6. QR sign-up lines (stage-flow.yaml pre-show and A6): cut, or rewrite as a forward-looking sign-up with no claim about today's screen.
7. Screen operator for A9 (OQ-11).
8. Real-GPU walkthrough today: laptop, projector, and a sighted check of all eight beats, then per-beat viewport-only screenshots eyeballed before commit.
9. S-E2 adversarial verdict once the branch exists; deploy-bar rows unchanged.

## 5. Verification loop (every commit)

App: `npm run typecheck && npm run build && npm run test` in `app/`. Rust (if touched):
`cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test` in `core/`. Cross:
`pwsh scripts/pii-scan.ps1`. Before the freeze: `pwsh scripts/check-all.ps1` 12/12,
browser walk, Codex review. Commits are atomic and conventional; agents stage only their
own paths in one breath (shared-checkout hygiene).

## 6. Design system

Every ATNI project uses the ATNI design system at `I:\ATNI_design-system` (revised
2026-09-12), digested in `docs/design/atni-design-system-digest.md`. Its source, fonts,
and logos never enter this public repo; values and rules do. Section 7 conflicts
(weights, body face, kind palette) stay with the human; this sprint applies only the
already-ruled title face (D-101) and the ground and text values.

## 7. Deferred (with reasons)

S-E4 Playwright smoke of the built form and S-E5 / S-R1 deploy-bar engineering (serve
only the live QR path, gated); form build switch (flag already exists,
`scripts/build-form.ps1`); D-100 `state_of_residence` beat (not in the spine's data
fields); need-to-solution routed-path beat (`queryPaths` exists, no A9 line calls for
it; post-convention wow slice W5); S-R3 term normalization; snapshot self-containment;
Codex re-review of `0cb27f0..d0e15a0` (after the convention); DDM contingency
screenshot (another tool's repo).
