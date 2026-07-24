# CODEX_GUIDE.md - Codex CLI Operating Manual

> Provenance note: the launch prompt instructed copying an existing Codex guide into
> this repo, but no source file exists on this machine (DECISIONS.md D-004). This
> guide was authored from the launch prompt's normative references to the guide's
> contents plus live `codex --help` and the config reference at
> https://developers.openai.com/codex/config-reference. If the original guide
> surfaces, reconcile; treat differences as a contract-doc contradiction
> (stop-the-line).

## 1. Operating model

Claude (director) owns judgment: architecture, contracts, phase sequencing, final
review calls. Codex CLI is the offload engine and sparring partner, invoked
headlessly via `codex exec`. The director never delegates: human gates, contract
interpretation, anything touching red data, or decisions that need project context
Codex does not hold. The director always applies final judgment to Codex output;
Codex findings are input, not verdicts.

Every offloaded task gets: a self-contained prompt (Codex has no session memory of
this project beyond what you pipe in), the relevant invariants from AGENTS.md, and
an explicit output contract (`--output-last-message <file>` at minimum; JSON schema
via `--output-schema` for structured extraction, contracts stored in `schemas/`).

## 2. SPECIALIZE blocks

### 2.1 Project, contract, live state
Project: Community Navigator (repo `community-connector`). Contract doc: `CLAUDE.md`
at repo root. Live-state file every session must trust over memory: `HANDOFF.md` at
repo root.

### 2.2 Red-data list
All real partner PII; everything on the predecessor exclusion list (CLAUDE.md
"Predecessor repo rules"); any credentials or tokens; any future real-community data
until the owner clears it in writing. **Nothing is currently cleared.** Green by
default: this repo's code, diffs, design docs, synthetic fixtures, logs from
synthetic runs.

### 2.3 Human-reserved contract gates (verbatim)
- Creating or adding any git remote; publishing or sharing anything outside the
  machine.
- Ingesting, transcribing, or fixturing any real person's data.
- Spending money: new paid services, APIs, or account upgrades.
- Committing to an external network protocol, identity standard, federation, or
  hosting vendor (designing the abstraction is yours; choosing the network is not).
- Choosing the project license.
- Breaking schema changes after v1.0 formats exist.
- Anything involving the old repo's git history or its PII remediation.

(2026-07-24 status: the license gate is resolved - PolyForm Noncommercial 1.0.0,
D-054 - and the remote/vendor/spend gates are conditionally opened for the single
D-053 intake path with push/deploy preconditions; see CLAUDE.md gate status notes
and HANDOFF.md. All other gates stand.)

### 2.4 Model pinning
Profiles live as `$CODEX_HOME\<name>.config.toml` files, selected with
`--profile <name>`. Pinned full ids (never aliases; see docs/ENVIRONMENT.md;
repinned 2026-07-17 per D-042 - gpt-5.6-sol family, full read/write granted):
- `grind`: model `gpt-5.6-sol`, effort low, sandbox danger-full-access, approval never.
- `review`: model `gpt-5.6-sol`, effort high, sandbox danger-full-access, approval never.
- `adversary`: model `gpt-5.6-sol`, effort high, danger-full-access, approval never
  (plan/design adversarial rounds; writes only to `C:\dev\_reviews\`).
Director side: this bootstrap session is Claude Fable, exact reported model id
`claude-fable-5`.

### 2.5 AGENTS.md invariant checklist
`AGENTS.md` I1-I12 is the review standard cited in every review prompt. Do not
paraphrase it into prompts; pipe or reference the file itself.

### 2.6 Approved offload tiers
Approved: code review of every non-trivial diff (the standing job); test authoring
from written specs; mechanical implementation from blueprints; bulk transforms and
lint burn-down; log and test-failure triage over piped output; structured extraction
over synthetic fixtures only, with `--output-schema` contracts stored in `schemas/`;
adversarial rounds on ADRs (two-round cap). Not approved: anything requiring project
judgment, anything touching red data.

### 2.7 Session sequence
Every session: (1) verify toolchain deltas against docs/ENVIRONMENT.md; (2) read
CLAUDE.md then HANDOFF.md then current-phase ADRs; (3) resume from HANDOFF.md's
"next actions", keeping Codex loaded with offload work in parallel.

## 3. Invocation recipes (Windows / PowerShell 7)

Piping into `codex exec` works in pwsh; prefer explicit temp files for large diffs
to avoid encoding surprises. Always capture output with `--output-last-message`.

Standing review recipe:
```powershell
# PowerShell - standing diff review
git diff HEAD | codex exec --profile review `
  "Review this diff against the invariants in AGENTS.md (repo root). Blocking issues first, then advisories. Be specific: file, line, failure mode." `
  --output-last-message .codex/review.md
```

Large-diff variant (temp file, avoids stdin size/encoding issues):
```powershell
# PowerShell
git diff HEAD | Out-File -Encoding utf8NoBOM .codex/diff.patch
codex exec --profile review "Review the diff in .codex/diff.patch against AGENTS.md invariants. Blocking first, then advisories: file, line, failure mode." --output-last-message .codex/review.md
```

Mechanical implementation from a blueprint:
```powershell
# PowerShell
codex exec --profile grind "Implement exactly the blueprint in docs/blueprints/<name>.md. Do not redesign. Run cargo test in core/ before finishing." --output-last-message .codex/grind-out.md
```

Structured extraction (synthetic data only):
```powershell
# PowerShell
codex exec --profile grind --output-schema schemas/codex/<contract>.schema.json "<task>" --output-last-message .codex/extract.json
```

`.codex/` is gitignored scratch space. Record the Codex session id (printed in exec
output) in the commit message for offloaded work (I11).

## 4. Deep-thinking escalation ladder

Rungs, in order; never skip downward:
1. **Patch** - the fix is local and obvious; apply it.
2. **Root cause** - the symptom recurs or the fix feels like whack-a-mole; stop and
   trace the actual causal chain before touching code again.
3. **Research** - the root cause implicates unfamiliar territory; one focused pass
   of reading (docs, source, prior art) before deciding.
4. **Redesign** - the design itself is the cause; write or amend an ADR, run an
   adversarial Codex round (two-round cap), then implement.

Triggers to climb: same bug twice; an estimate blown 2x; a fix that requires
touching more than one architectural stance; disagreement between contract docs;
any change to a `schemas/` format. Timeboxes: rungs 2-3 get one focused pass each -
if a pass ends without resolution, climb, do not repeat the pass. Every climb past
rung 1 gets a DECISIONS.md entry: trigger, causal chain, options, choice, strongest
surviving objection. Phase 1 work is presumptively rung 3-4.

## 5. Cost discipline

Report Codex usage to the human in HANDOFF.md at roughly 50% of whatever session
budget applies. Prefer `grind` wherever the task is mechanical; `review` is for
judgment-adjacent reading, not bulk work. A 2x estimate blowout is a ladder trigger
(section 4), not a reason to grind harder. Batch small mechanical tasks into one
exec call when they share context.

## 6. Usage failover (human directive, 2026-07-06)

If Claude usage limit reaches ~98%: offload active mechanical work to `grind`,
verification to `review`, park director-judgment work in HANDOFF.md, resume as
director when the limit resets. Mark commits made in this mode `[codex-offload]`.

## 7. Standing routing policy (token conservation)

Adopted 2026-07-06 from the Codex analysis of the design-research workflow
(docs/analysis/token-analysis-2026-07-06.md; DECISIONS.md D-009). Claude and
Codex draw on separate budgets: spend Claude on deciding what matters, Codex on
collecting, transforming, and drafting. Compress every cross-stage handoff
before it re-enters Claude.

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
| Claude workflow agents | Rare | Only when Claude-specific parallel judgment is worth it, with hard output caps |
| Commit readiness and final acceptance | Claude director | Read Codex outputs, decide, commit |

Known pitfall: if a Codex task writes an artifact file, point
`--output-last-message` at a DIFFERENT path - the last-message write clobbers
same-path artifacts (it cost us a recovery-from-log this session).

## 8. Degraded mode

If Codex CLI is missing, unauthenticated, or persistently failing: proceed with
self-review, mark affected commits `[unreviewed-by-codex]`, log the degradation and
its start point in HANDOFF.md for the human, and do not stall. Exit degraded mode
by re-running the section 2.7 sequence and noting recovery in HANDOFF.md.
