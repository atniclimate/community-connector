# NEXT_SESSION.md - Launch cards (refreshed 2026-09-14, end of the convention sprint)

> Reading order: CLAUDE.md -> HANDOFF.md -> this file. Sprint plan of record:
> `docs/planning/CONVENTION-SPRINT-2026-09-14.md`; post-convention plan:
> `docs/planning/NEXT-PHASE-convention-e2e.md`. Previous briefs live in git history.

## The 60-second brief

The A9 constellation is built and verified on synthetic data. `pwsh scripts/reveal.ps1`
opens the app on the ATNI convention group; Present, then `Home` and Space through eight
beats: overview, members, committees, organizations, connectors, one node (the spotlight
person lit by opacity with Energy, Climate Resilience, Housing, and Native Vote named
beside it, camera holding), shared priorities (48 Relatives lit, no labels), and the
finale (60 Relatives, 15 committees, 12 organizations, lit whole, held). No person name
ever renders on stage; the rail is hidden until `r`. The cue sheet is in `CONTROLS.md`:
three Space presses across the last three sentences of T32, starting at "communities
connected in new ways". check-all is 12/12 at HEAD, the app suite is green, three Codex
rounds are dispositioned (D-103 addenda). What no agent could do: see it on the real GPU
and the projector. That is the human's first action today. S-E2 (edges at approval) is
built and reviewed on branch `s-e2-intake-edges`, not merged; its verdict is the human's.
The session slot is Tuesday 2026-09-15 at about 11:45.

## Launch card: real-GPU walkthrough support (NEXT, with the human present)

**Gate** the human's eyes - **Budget** ~20k tokens

```powershell
Set-Location I:\community-connector
claude --model claude-sonnet-5 --effort medium
```

> Role: stagehand. Run `pwsh scripts/reveal.ps1`, hand the human the cue sheet in
> `CONTROLS.md`, step the eight beats with them, note anything that reads wrong from the
> back of a room (caption size, spotlight visibility, fit padding, label collisions), and
> fix only beat JSON or CSS values; any render-path change waits for a plan. Capture
> viewport-only screenshots per beat, have the human eyeball each, commit them to
> `docs/design/screenshots/reveal-<beat-id>-2026-09-14.png`. If the human answers any of
> the sprint plan's section 4 one-liners, record them in DECISIONS.md as D-104.

## Launch card: S-E2 verdict and merge (AFTER the convention)

**Gate** human verdict recorded in DECISIONS.md, then check-all 12/12 on main -
**Budget** ~60k tokens

```powershell
Set-Location I:\community-connector
claude --model claude-fable-5-1 --effort high --permission-mode plan
```

> Role: integrator. Read `docs/blueprints/intake-edges.md` on the branch (its adversarial
> round section), D-103 addendum 2, and the human's verdict. Merge `s-e2-intake-edges`
> into main (one import-line conflict expected in `app/src/ui/intake/panel.ts`; union of
> both import lists), regenerate the fixture so the embedded template carries 0.1.2, run
> the full loop, remove the worktree. Do not merge without the recorded verdict.

## Launch card: deferred advisories and S-E1 remainder (post-convention, when section 7 is answered)

**Gate** check-all 12/12 - **Budget** ~80k tokens

> Role: frontend engineer. Beat-sheet `schema_version` envelope with the load-time
> validator already in place; split presenter coordination out of `app/src/viz/index.ts`;
> wiring-level tests for hover suppression, `handlePresenterKeydown`, and the rail; then
> the section 7 typography and kind-color decisions applied through the token pipeline.
