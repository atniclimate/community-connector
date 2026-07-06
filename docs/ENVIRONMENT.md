# ENVIRONMENT.md - Toolchain and Pinned Models

Recorded 2026-07-06 on ATNI-PATRICK (Windows 11 Pro 25H2, build 26200; 13th Gen
i5-1340P, 16GB RAM, Intel Iris Xe iGPU - also the performance-target hardware).
Primary shell: PowerShell 7. Re-verify this file whenever a tool is upgraded.

## Toolchain

| Tool | Version | Notes |
|---|---|---|
| git | 2.55.0.windows.2 | repo-local identity set; no global identity on machine |
| node | v24.14.1 | |
| npm | 11.12.1 | npx same |
| rustup | 1.29.0 | |
| rustc / cargo | 1.96.1 | 2026-06-26 toolchain |
| wasm-pack | 0.15.0 | installed via `cargo install wasm-pack --locked` this session |
| codex CLI | codex-cli 0.142.5 | authenticated (ChatGPT login) |

## Director model

This session reports model id: **claude-fable-5** (Claude Fable 5, Claude Code CLI).
If Fable access closes, the strongest available Claude model inherits the director
role and contract unchanged.

## Codex pinned profiles

Profiles are separate files at `$CODEX_HOME\<name>.config.toml`
(`C:\Users\PatrickFreeland\.codex\`), selected with `codex exec --profile <name>`.
Full model ids are pinned deliberately - alias drift burned the predecessor project.

| Profile | Model (full id) | Effort | Sandbox | Approval | Use |
|---|---|---|---|---|---|
| grind | gpt-5.4-mini | low | workspace-write | never | mechanical implementation, bulk transforms, lint burn-down |
| review | gpt-5.5 | high | read-only | never | diff review, adversarial ADR rounds, triage |

Source: https://developers.openai.com/codex/models (fetched 2026-07-06).
gpt-5.5 is the current default/strongest recommended; gpt-5.4-mini is the documented
fast/low-cost tier for mechanical subagent work. gpt-5.2 and gpt-5.3-codex are
deprecated - never fall back to them. Re-verify ids after any codex CLI update.

## Frontend toolchain

Recorded at scaffold time. Vite stays 7.x (vite-plugin-singlefile compatibility -
do not upgrade to v8 without checking the plugin). TypeScript strict mode with
noUncheckedIndexedAccess and exactOptionalPropertyTypes.

| Package | Version |
|---|---|
| vite | ^7.3.6 |
| typescript | ^6.0.3 |
| vite-plugin-singlefile | ^2.3.3 |
| 3d-force-graph / three | not yet added (Phase 3) |
