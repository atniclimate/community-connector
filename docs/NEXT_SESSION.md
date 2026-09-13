# NEXT_SESSION.md - Launch cards (refreshed 2026-09-12, evening true-up)

> Reading order: CLAUDE.md -> HANDOFF.md -> this file. Session plan of record:
> `SESSION_ROSTER.yaml`; next-phase plan: `docs/planning/NEXT-PHASE-convention-e2e.md`.
> Previous briefs live in git history of this file.

## The 60-second brief

The reveal works. `pwsh scripts/reveal.ps1` opens the dev app on the synthetic ATNI
convention group (60 people, 15 committee rings, 12 orgs). There, presenter mode steps
through beats that fit the camera, dim everything off-beat, and label only what matters.
check-all has been 12/12 since the viz work; `main` is pushed to origin. The
human's next ask is the full loop: QR -> ATNI-branded dark-mode form -> approved entries
land with real edges -> presenter keys `c`/`o`/`t`/`s` plus a button rail -> a scripted
end-to-end test. Two findings shape it. An approval creates a node but no edges
(`approval.rs:602-624`), so S-E2 is new permission-adjacent work. The deploy bar and the
real-data gate are unchanged, so everything runs on synthetic data and going live is the
human's decision (D-090c, D-099). Convention: 2026-09-14.

## Launch card: S-R0 remainder - toolchain pin + troika dependency (NEXT, tiny)

**Gate** `pwsh scripts/check-all.ps1` 12/12 - **Budget** ~10k tokens

```powershell
Set-Location I:\community-connector
claude --model claude-sonnet-5 --effort medium
```

> Role: fixer. Two mechanical fixes, one conventional commit each, check-all green before
> each. (1) Add `rust-toolchain.toml` at the repo root pinned to the cargo/rustc version
> check-all is green on (read `cargo --version` first; do not guess). (2) Move
> `troika-three-text` from devDependencies to dependencies in `app/package.json` without
> changing its pinned version (`app/src/viz/labels.ts` imports it); update the lockfile
> with `npm install` in `app/`. Scope fence: those two files plus `app/package-lock.json`.
> Stop when both commits exist and SESSION_ROSTER.yaml S-R0 is marked done.

## Launch card: S-E3 - presenter controls (keys + rail, with W1 and the W3 name gate)

**Gate** `npm run typecheck && npm run build && npm run test` (app/), check-all 12/12,
headless screenshots of each new beat, then the human visual gate - **Budget** ~80k tokens

```powershell
Set-Location I:\community-connector
claude --model claude-sonnet-5 --effort high --permission-mode plan
```

> Role: frontend engineer (plan first). Outcome: in presenter mode, `c` committees, `o`
> organizations, `m` members jump to kind beats; `t` shows an exact-match tribe beat;
> `s` shows members by state of residence (D-100: add an optional `state_of_residence`
> enum to the atni-convention person kind - US states plus DC, "Outside the United
> States", "Prefer not to say", group visibility, schema PATCH bump - give the fixture
> generator synthetic states, and make the beat aggregate-only with states under 3
> shown as "fewer than 3"; widen the scope fence to `fixtures/`, `schemas/` if the
> bump needs it, and `app/scripts/generate-atni-ops.mjs`); a small on-screen rail mirrors the keys (keyboard-reachable, ARIA-labeled,
> holds at 375 px, no new animation, reduced-motion safe). Captions carry projection
> counts ("15 standing committees"). Add the `nameOnStage` beat flag (default false):
> when false, measure beats show "N highlighted" and no person labels.
> Scope fence: `app/src/viz/` (index.ts `handlePresenterKeydown`, presenter.ts),
> `app/src/ui/ui.css`, `app/public/beats.atni.json`, `CONTROLS.md`, tests. State changes
> only through `app/src/state` actions (I4). No core changes, no new dependency.
> Read first: SESSION_ROSTER.yaml S-E3; `docs/planning/NEXT-PHASE-convention-e2e.md`
> sections 2(d), 3 (S-E3), and the wow slice (W1, W3); `docs/blueprints/presenter-mode.md`.
> Visual check: headless Chromium with `--use-angle=swiftshader` captures the WebGL
> canvas (see D-097); drive `http://localhost:<port>/?group=atni-convention`.
> Stop when the gate passes, screenshots are sent to the human, and S-E3 is marked
> partial pending the human visual gate.

## Launch card: Codex review of the post-review viz commits

```bash
# From Claude Code (Bash tool), via the codex-adversary skill wrapper; target range
# 0cb27f0..d0e15a0 (frustum fit, group param, kind beats, emphasized labels, rings).
```

> Use the `codex-adversary` skill; prompt it to read only that commit range with `git show`,
> apply AGENTS.md and ADR-004 as criteria, and write to
> `C:\dev\_reviews\community-connector\<date>_viz-reveal-diff-review.md`. Verify every
> High finding on disk before fixing.

## Mid-session rules (all cards)

- Skipped a file or test: raise effort one notch, stay here.
- Two failed corrections on one bug: `/clear`. Context past ~130k: write the handover into
  HANDOFF.md "next actions", `/clear`, split the task.
- Shared checkout: stage and commit only your own paths, in one breath.

## Questions for the human (one-line answers unblock the roster)

1. Can you share the ATNI palette and font files (with licenses) for the dark-mode form?
2. Convention day: synthetic demo only, or pursue the go-live checklist (D-090c)?

Answered 2026-09-12: `s` = state of residence (D-100); push authorized and done.
