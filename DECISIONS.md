# DECISIONS.md - Judgment Calls, Ladder Climbs, Adversarial Outcomes

Newest entries at the bottom. Every deep-thinking ladder climb past rung 1, every
adversarial Codex round outcome, and every nontrivial autonomous decision lands here.

---

## D-001 (2026-07-06) - Product name vs folder name

The brief names the product **Community Navigator** but mandates the folder
`C:\dev\community-connector`. Proceeding with product name Community Navigator in
docs and package names, folder as mandated. **Human may rename either later;**
flagged in HANDOFF.md. No code depends on the folder name.

## D-002 (2026-07-06) - Default branch `main`

`git init -b main`. The predecessor used `master` with a configured default of
`main`, which caused tooling friction. Reversible; standard.

## D-003 (2026-07-06) - Git identity set deliberately, pending confirmation

Predecessor commits auto-resolved to `patrick@atni57.onmicrosoft.com` (a machine
identity nobody chose - exactly the failure the brief flags). This repo sets
in-repo config: `Patrick Freeland <accounts@indigenousaccess.org>` (the account
identity of this machine's Claude session). **Human gate-adjacent: confirm or
correct in HANDOFF.md.**

## D-004 (2026-07-06) - docs/CODEX_GUIDE.md reconstructed, not copied

The launch prompt says to copy the Codex guide into the repo, but no source
CODEX_GUIDE.md exists anywhere under C:\dev or the user's .claude directory
(verified by glob). The guide was authored fresh from the launch prompt's
references to its contents (operating model, SPECIALIZE blocks, escalation ladder
as its section 4, cost discipline, recipes) plus current `codex --help` output.
If the human has the original guide, drop it in and reconcile; differences should
be treated as contract-doc contradiction (stop-the-line rule).

## D-005 (2026-07-06) - Usage-limit failover rule (human directive)

Mid-session human directive: when Claude usage limit reaches ~98%, send the job to
Codex until the limit resets, then resume. Encoded in CLAUDE.md "Usage failover".
Mechanically: mechanical/implementation work goes to the `grind` profile,
verification to `review`; director-level judgment work parks in HANDOFF.md rather
than being delegated, because Codex does not hold the contract context.

## D-006 (2026-07-06) - Design research run as a parallel multi-agent workflow

Per human directive, a 5-researcher + synthesis + adversarial-critique workflow
(run id wf_afb4e38d-a45) produced `docs/design/DESIGN_BRIEF.md` to drive Phase 3
visual/motion direction. The brief is advisory input to Phase 3, not a contract
doc; conflicts with invariants resolve in favor of AGENTS.md.
