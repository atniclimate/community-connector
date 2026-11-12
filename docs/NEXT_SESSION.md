# NEXT_SESSION.md - Launch cards (refreshed 2026-11-11, close of the relatives session)

> Reading order: CLAUDE.md -> HANDOFF.md -> this file. Sequence of record:
> `docs/planning/RELATIVES-ACROSS-GRAPHS-2026-11-11.md` section 9; decision D-107.
> Previous briefs live in git history.

## The 60-second brief

On 2026-11-11 the owner named the primary goal: a web of relatives that can find each
other across institutions, each person at the center of their own view, each entity
sharing as much or as little as it chooses, consenting entities bridging graphs. The
convention work is the on-ramp. The session fixed four core defects (one changes
EdgeCreate: members are now denied `target_missing` for any endpoint outside their
projection, archived entities included) and built the first person-centered slice on
the synthetic relatives-demo group: record mirror, analysis scope, continuance,
introductions, routing reachable from the app, a Relatives panel, and a "My relatives"
ring layout. Try it with `npm run dev` in `app/` and `?group=relatives-demo`. The TSDF
consent link is built. check-all is 12/12 at `130e20d`; form 81/81, relay 47/47. Nobody
has seen the new UI in a browser, and its copy is draft (D-051, D-023). ADR-008
(two-sided relations) and ADR-007 (Network circle and bridging) are Proposed and
reviewed; acceptance is the human's. The gap that matters most: approved remote
respondents land unowned (ADR-005 D7) and viewers are asserted (D-015), so none of this
reaches a real person yet. The convention outcome is unrecorded. Nothing was pushed.

## Launch card: ADR-008 implementation, owner claim, key-bound viewer (NEXT, after the human accepts ADR-008)

**Gate** ADR-008 accepted by the human (recorded in DECISIONS.md); ADR-008 test
obligations green; check-all 12/12 - **Budget** ~150k tokens

```powershell
Set-Location I:\community-connector
claude --model claude-fable-5-1 --effort high --permission-mode plan
```

> Role: core engineer. Must land before the S-E2 branch or any form answer creates
> person-to-person edges. Read ADR-008 in full, including its review log and test
> obligations (golden captures first). Build in order: the template `subject` field per
> kind; `EdgeAcknowledge` authorized only for the endpoint owner and only for an edge in
> their projection; Confirmed and Standing status in cn-perm; final private refusal that
> looks like an unanswered ask to everyone but the refuser; grandfathering by
> `edge.schema_version` in synthetic groups only, capped at Group; model 0.1.2; fixture
> regeneration. Then, in the same slice and with its own short design note: an owner
> claim so an approved respondent's record can belong to them, and a viewer bound to a
> key instead of asserted. Synthetic fixtures only. Do not touch `s-e2-intake-edges`.

## Launch card: ADR-007 Phase A (after ADR-008 lands and the human accepts ADR-007)

**Gate** ADR-007 accepted and its open question 5 (TSDF T2 mapping and reviewed T2
wording) answered; Phase A behind its build flag; check-all 12/12 - **Budget** ~150k tokens

```powershell
Set-Location I:\community-connector
claude --model claude-fable-5-1 --effort high --permission-mode plan
```

> Role: permission engineer. Read ADR-007 D1-D4, D7, D8, the test obligations, and the
> review log. Add required `now` and `use` to `project` and `export_projection` and let
> the compiler list every caller; agreements naming outside persons with (kind, attr)
> scopes; admission per (agreement, use) up to T2 as if at Group, never T3; bridge
> consent; caches keyed on (viewer, use). Unowned person records, intake records, and
> relation-bearing attributes never cross. Receipts (D9) are a later slice. Model 0.1.3.
> No real data, no transport, no Phase B.

## Launch card: I9 walkthrough and copy review (any time, with the human present)

**Gate** the human's eyes - **Budget** ~30k tokens

```powershell
Set-Location I:\community-connector
claude --model claude-sonnet-5 --effort medium
```

> Role: stagehand. Run the dev app with `?group=relatives-demo`; walk the Relatives
> panel and the ring toggle by keyboard, at 375px, with a screen reader and with reduced
> motion; check the TSDF link on the built form and the in-app consent path. Fix only
> CSS or markup that fails; collect the human's wording rulings for D-051 and D-023 and
> record them in DECISIONS.md.

## Launch card: S-E2 verdict and merge (after the human's verdict)

**Gate** human verdict recorded in DECISIONS.md; ADR-008 landed first; check-all 12/12
on main - **Budget** ~60k tokens

> Role: integrator. Re-check the branch against the EdgeCreate rule change (`23aacec`)
> and ADR-008, merge (one import-line conflict expected in `app/src/ui/intake/panel.ts`;
> union), regenerate the ATNI fixture so its embedded template carries 0.1.2. Do not
> merge without the recorded verdict.
