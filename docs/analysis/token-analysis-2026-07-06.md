# Token Analysis - Design Research Workflow

## Where the Claude tokens went

The workflow spent tokens in the shape of the orchestration, not just in model reasoning. It used 7 Claude agents: 5 parallel researchers, 1 synthesis agent, and 1 adversarial critique agent that failed at the usage limit. The run produced 460,019 subagent output tokens, 112 tool uses, and about 12 minutes of wall clock.

Measured avoidable multipliers:

- Repeated context: the shared project context was 1,623 characters and was copied into every researcher prompt, then again into synthesis. The five researcher prompts totaled 14,328 characters before tool output.
- Broad overlapping lanes: visual language, motion, UI chrome, and perf-a11y all researched adjacent WebGL, labels, bloom, theming, and accessibility topics. This created useful cross-checks, but also repeated source discovery and repeated explanations.
- Verbose schema output: each research agent returned a rich object with summary, techniques, antipatterns, perf notes, and source arrays. The five final research payloads were 147,696 characters when re-serialized for synthesis.
- Full JSON re-serialization: synthesis embedded all research as `JSON.stringify(found, null, 1)`, making the synthesis prompt about 151,497 characters, roughly 37,875 tokens before any synthesis output.
- Oversized synthesis artifact: the design brief result was about 37,097 characters, roughly 9,300 visible-output tokens.
- Critique prompt duplication: the failed critique prompt embedded the whole brief, another 37,862 characters, roughly 9,466 tokens, before any critique output.
- Tool-use churn: 112 tool uses across five researchers means Claude paid for browsing, fetch summaries, local file reads, and the final prose. That is exactly the class of work Codex should absorb when Codex is a separate budget.

The unavoidable spend was the director-level framing: constraints, privacy boundaries, decision criteria, and final acceptance. The avoidable spend was running broad research, local audit, synthesis drafting, and adversarial critique as Claude workflow agents instead of Codex jobs with capped outputs.

## What should move to Codex

Use Claude director for scope, gates, and final judgment. Use Codex for the bulk work and force small contracts between stages.

1. Research harvest - Codex review

```powershell
codex exec --profile review "Research current visual, motion, UI chrome, WebGL performance, and canvas accessibility patterns for the Community Navigator 3D graph. Read CLAUDE.md, AGENTS.md, and docs/CODEX_GUIDE.md first. Return one Markdown memo under 1800 words with sources, concrete parameters, and open tradeoffs. Do not touch red data." --output-last-message .codex/design-research-harvest.md
```

2. Predecessor frontend audit - Codex review

```powershell
codex exec --profile review "Audit only the allowed predecessor frontend files listed in .codex/predecessor-audit-allowlist.md. Do not open excluded data paths. Return transferable techniques and antipatterns under 1200 words, with file references only." --output-last-message .codex/predecessor-audit.md
```

3. Compression before synthesis - Codex grind

```powershell
codex exec --profile grind "Read .codex/design-research-harvest.md and .codex/predecessor-audit.md. Produce .codex/design-brief-input.md as a dense synthesis input under 2500 words: decisions, parameters, risks, and citations. No new research." --output-last-message .codex/design-compress.md
```

4. Candidate design brief - Codex grind, Claude final edit

```powershell
codex exec --profile grind "Draft docs/design/DESIGN_BRIEF.candidate.md from .codex/design-brief-input.md. Preserve AGENTS.md invariants, use hyphens only, include explicit reduced-motion and 375px requirements, and keep each checklist item implementable." --output-last-message .codex/design-brief-draft.md
```

5. Adversarial critique - Codex review

```powershell
codex exec --profile review "Critique docs/design/DESIGN_BRIEF.candidate.md against AGENTS.md and the constraints in CLAUDE.md: Iris Xe feasibility, 5MB snapshot, data-driven theming, accessibility, privacy, and hyphen-only docs. Blocking findings first." --output-last-message .codex/design-brief-critique.md
```

Claude director should still choose the final aesthetic direction, resolve tradeoffs, decide whether a critique finding blocks Phase 3, update HANDOFF.md and DECISIONS.md, and make commits.

## Standing routing policy

| Task type | Route | Token rule |
|---|---|---|
| Human gates, contract interpretation, phase sequencing, final architecture calls | Claude director | Never delegate |
| Real or possibly real PII, credentials, predecessor excluded data | Nobody | Stop the line |
| Broad web research over public sources | Codex review | Cap output to 1200-1800 words |
| Local codebase audits over green repo files | Codex review | Give allowlist and exact questions |
| Mechanical implementation from a written blueprint | Codex grind | Workspace-write, run checks, no redesign |
| Test authoring, fixtures with synthetic data, lint burn-down | Codex grind | Batch related work |
| Synthesis first drafts from bounded notes | Codex grind | Use compressed inputs, not raw journals |
| Diff review, adversarial ADR or design critique | Codex review | Blocking findings first |
| Claude workflow agents | Rare | Use only when Claude-specific parallel judgment is worth the token cost, with hard output caps |
| Commit readiness and final acceptance | Claude director | Read Codex outputs, decide, commit |

## Estimated Claude-token savings for this run

Conservative policy savings: 380k-420k Claude tokens. That assumes the director still spent Claude tokens writing the task specs, reading compressed Codex outputs, and doing final judgment, while all five research lanes, predecessor audit, synthesis draft, and critique moved to Codex.

Aggressive policy savings: 430k-450k Claude tokens. That assumes a single Codex research harvest plus one Codex audit replaced the five Claude researchers, and the director read only a 2,500-word compressed synthesis input plus a candidate brief and critique.

The practical rule: spend Claude on deciding what matters, not on collecting, reformatting, and re-reading large research blobs. Codex should absorb the bulk tokens, and every cross-stage handoff should be compressed before it re-enters Claude.
