# HANDOFF.md - Live State

> This file outranks session memory. Reading order for a new session: CLAUDE.md,
> then this file, then current-phase ADRs.

Last updated: 2026-07-06, end of session 1 (bootstrap). Session ended early on a
Claude usage limit (resets 3:10am America/Los_Angeles); remaining mechanical work
was delegated to detached Codex processes per the usage-failover rule.

## Current phase and criterion status

**Phase 0 - Bootstrap: ~90% complete.**

| Criterion | Status |
|---|---|
| Repo exists with Section 3 skeleton | PARTIAL - contract docs, scripts, schemas/fixtures dirs done by director; cargo workspace + app shell delegated to Codex (see Delegations) |
| Contract docs authored | DONE (CLAUDE.md, AGENTS.md, DECISIONS.md, docs/CODEX_GUIDE.md, docs/ENVIRONMENT.md, docs/LAUNCH_PROMPT.md) |
| PII tripwire installed and demonstrably firing | DONE - planted fake email+phone, hook blocked commit with exit 1, violation removed; hook is tracked at scripts/hooks/pre-commit via core.hooksPath |
| Toolchain verified and recorded | DONE (docs/ENVIRONMENT.md; wasm-pack 0.15.0 installed this session) |
| First commits made | DONE (see Commits) |
| Codex pinned + round-trip confirmed | PARTIAL - profiles pinned (grind=gpt-5.4-mini, review=gpt-5.5); first ping hung before the project dir was trusted in codex config.toml; trusted entry added; retry was in flight at session end (.codex/ping.md should contain PONG) |

Phase 1 not started. The design brief for Phase 3 already exists (see Design).

## Commits this session

- 1c1ae52 chore: add PII tripwire, git hooks, and size budget check
- 47902f7 docs: author contract docs, codex guide, environment record; archive launch brief
- (uncommitted at handoff time: docs/design/DESIGN_BRIEF.md, this file, DECISIONS.md updates - commit them at session start if session 1 could not)

## Delegations in flight (Codex, detached processes)

Both launched via Start-Process so they survive this session; logs in .codex/
(gitignored). Task specs are self-contained files:

1. **Scaffold** - `.codex/task-scaffold.md` -> profile grind (gpt-5.4-mini),
   workspace-write, network on. Creates core/ cargo workspace (8 crates + cn CLI),
   app/ Vite+TS strict shell, schemas/fixtures/docs-adr READMEs; runs cargo
   fmt/clippy/test, npm typecheck/build/build:snapshot, pii-scan. Output:
   `.codex/scaffold-result.md`, log `.codex/scaffold-log.txt`. Codex was told:
   NO git commands; director reviews and commits.
2. **Token analysis** - `.codex/task-token-analysis.md` -> profile review
   (gpt-5.5), read-only. Analyzes the design-research workflow (script + journal
   paths inside the task file) for token conservation and a standing
   Claude/Codex routing policy. Output: `.codex/token-analysis.md`.

If either output file is missing at resume: check the matching *-log.txt, then
re-run `codex exec --profile <p> "Implement exactly the task in .codex/<file>" --output-last-message .codex/<out>` from the repo root.

## Design (parallel directive from the human)

A 7-agent Claude workflow (run id wf_afb4e38d-a45) produced
`docs/design/DESIGN_BRIEF.md` (Hearthlight Constellation direction: tokenized
visual system, motion system with reduced-motion variants, data-driven theming
contract, Iris Xe performance budget, a11y mechanisms, Phase 3 checklist).
**The adversarial critique agent died on the usage limit, so the brief is
UNCRITIQUED.** Run the critique via codex review profile at resume (see Next
actions). Workflow is resumable with cached results:
Workflow({scriptPath: "C:\Users\PatrickFreeland\.claude\projects\C--dev-CPF-RCN-demo\e973f411-4d17-4215-bafa-02563f3f9f32\workflows\scripts\design-research-wf_afb4e38d-a45.js", resumeFromRunId: "wf_afb4e38d-a45"}).

## Open human gates (one-line answers suffice)

1. Git identity: commits use `Patrick Freeland <accounts@indigenousaccess.org>` - confirm or give the identity to use.
2. Naming: product "Community Navigator" in folder `community-connector` - keep both, or rename one?
3. License: none chosen (gate) - fine to leave unlicensed for now?
4. No git remote exists and none will be added without instruction (standing gate, no action needed).

## Degraded modes / warnings (do not lose these)

- DESIGN_BRIEF.md is uncritiqued (see Design) and unverified against I10
  (em dashes) - scan and fix before treating it as Phase 3 input.
- Cron resume is SESSION-ONLY: the scheduled 3:12am job fires only if this
  Claude Code terminal stays open. If it was closed, start a session in
  C:\dev\community-connector and say "resume from HANDOFF.md".
- First codex exec hung until `[projects.'c:\dev\community-connector']
  trust_level = "trusted"` was added to codex config.toml - if codex hangs
  again, check trust config first.
- The usage-failover rule (CLAUDE.md) is ACTIVE as of this handoff.

## Next actions for the successor session (in order)

1. Verify Codex scaffold: read `.codex/scaffold-result.md`, `git status`; run the
   full verification loop yourself (core/: cargo fmt --check, clippy -D warnings,
   test; app/: npm run typecheck, build, build:snapshot; root: pwsh
   scripts/pii-scan.ps1); commit as small conventional commits including the
   Codex session id from `.codex/scaffold-log.txt` (invariant I11). Commit any
   leftover session-1 docs first.
2. Read `.codex/token-analysis.md`; fold its routing policy into
   docs/CODEX_GUIDE.md section 5 and log a DECISIONS.md entry.
3. Adversarial round on docs/design/DESIGN_BRIEF.md via
   `codex exec --profile review` (attack: Iris Xe feasibility, 5MB bundle, theming
   contract, a11y gaps, I10); revise, commit.
4. Begin Phase 1: draft ADR-001 (domain model), the two contrasting synthetic
   group templates in fixtures/ (all emails @example.test), then the ADR-001
   adversarial round (two-round cap). ADR-002, ADR-003 follow.
