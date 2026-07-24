# community-connector - External Dependency Audit

(audit note 2026-07-24: this is the world-readable revision of the 2026-07-11
read-only audit, produced by the D-055 pre-publish sweep. Machine-local
operational detail - backup topology, mirror and restore procedure, absolute
local paths, and exact machine measurements - was split out of this document
and lives outside the tracked tree in `_private/`, gitignored. This doc is
subordinate to HANDOFF.md and DECISIONS.md. Docs use hyphens, never em dashes,
per CLAUDE.md and AGENTS.md I10.)

Scope: every reference in the repo to a file or directory outside the project
root, verified and classified. Installed programs and toolchains are out of
scope except the single Runtime Notes paragraph. Fixtures are synthetic; no
file contents are quoted beyond what path classification needs.

## Summary

- No reparse points (junctions/symlinks) and no `.lnk` shortcuts anywhere in
  the tree.
- No `.env` / `.env.*` files present; no secrets to redact.
- Remotes: at the original 2026-07-11 audit the repository had no git remotes
  (D-026, an accepted single-machine risk). On 2026-07-24 the human
  conditionally opened the remote/publishing gate for exactly one path - the
  public remote `atniclimate/community-connector` - with preconditions before
  any push: license in-repo, the D-055 pre-publish sweep passed, and core
  stability (D-053/D-055; CLAUDE.md gate-status notes). All other gates stand
  unchanged.
- Every candidate external path was verified at audit time: all exist. Zero
  broken references.
- All build and runtime inputs are either in-project or fetched by a package
  manager (crates.io, npm registry). Nothing outside the root is consumed by a
  build or at runtime. The only external references are documentary (sibling
  repos named in prose).
- The app imports the wasm-pack output `core/crates/cn-wasm/pkg/cn_wasm.js`,
  which is in-project but gitignored build output. It is not an external
  dependency; it is a restore-time rebuild step
  (`wasm-pack build crates/cn-wasm --target web`).

## Findings

Absolute local paths are intentionally omitted; sibling repositories are named
only where those names already appear in tracked docs.

| reference | referenced from (file:line) | kind | impact | remediation |
|---|---|---|---|---|
| The predecessor repo (read-only reference; PII exclusion rules in CLAUDE.md) | CLAUDE.md "Predecessor repo rules", docs/LAUNCH_PROMPT.md:51 | doc-prose | docs-only | DECLARE - read-only predecessor reference; source of ported concepts and isolated frontend techniques. Never bulk-copied; never consumed by build or runtime. |
| TSDF (TieredSovereignDataFramework) sibling repo | DECISIONS.md:328 | doc-prose | docs-only | DECLARE - the Tiered Sovereign Data Framework standard that this project's provenance/tiering (R10, TSDF T1) aligns to. Referenced conceptually; not consumed by build/runtime. |
| Sibling repos cap-assessor, TCR-policy-scanner, GeoBase, engagement-database | DECISIONS.md:397-398, docs/design/integration-plan-2026-07-06.md:37-39/64/380-388, docs/NEXT_SESSION.md:20, docs/PROJECT_PLAN.md:193-194, docs/research/graph-networks-report-2026-07-06.md:749-776, HANDOFF.md | doc-prose | docs-only | DECLARE-adjacent / IGNORE - four named FUTURE integration targets, explicitly "integrate after the pilot" (integration-plan:37). Not consumed by any current build or runtime. |
| Codex CLI configuration under `$CODEX_HOME` | docs/ENVIRONMENT.md:28 | doc-prose / config | docs-only | IGNORE - user-profile location of Codex CLI pinned-profile TOMLs, indirected via `$CODEX_HOME`. Codex is an optional offload engine, not a build/runtime input. Distinct from the in-project gitignored `.codex/` scratch dir. |
| Workspace-parent narrative mentions | DECISIONS.md:31/398, docs/LAUNCH_PROMPT.md:150/158/432 | doc-prose | docs-only | IGNORE - narrative mentions of the development workspace parent and repo-creation history. No specific file consumed. |
| `ISDGraph:\ATNI-Climate` | docs/THE_STORY.md:82 | doc-prose | docs-only | IGNORE - a fictional federated-address URI in narrative, not a filesystem path. |
| `http://example.com`, `https://polyformproject.org/...` | LICENSE.md:22, LICENSE.md:3 | doc-prose | docs-only | IGNORE - license template placeholder URL and license text URL, not file paths. |
| `github.com/atniclimate/pnw-tribal-dashboard` (+ "related repos") | docs/THE_STORY.md:105 | doc-prose (external repo URL) | docs-only | IGNORE - narrative citation of an external GitHub repo, not fetched/consumed by build or runtime. All other external URLs in the tree are documentary citations (research report + design docs); the app runtime fetches only same-origin `/fixtures/...`. |
| `std::env::temp_dir()` | core/crates/cn-store/tests/blueprint.rs:539 | code (test) | docs-only | IGNORE - a test writes to the OS temp dir; transient, no fixed external path. |

Intra-project relative references that look path-shaped but resolve INSIDE the
root (verified; not findings, listed for completeness):

- `app/scripts/smoke-node.mjs:8`, `app/scripts/generate-demo-ops.mjs:6` -
  `path.resolve(here, "../..")` -> repo root.
- `app/smoke/smoke.ts:2`, `app/src/wasm/worker.ts:1` - `../../core/crates/cn-wasm/pkg/cn_wasm.js`
  -> in-project wasm-pack output (gitignored; rebuild step).
- `app/src/theme/theme.test.ts:3-5`, `app/src/viz/viz.test.ts:37` - `../../../fixtures`,
  `../../../schemas` -> in-project.
- `core/crates/cn-api/tests/measure.rs:13`, `core/crates/cn-schema/tests/blueprint.rs:11/13` -
  `include_str!("../../../../fixtures/templates/...")` -> in-project.
- `app/vite.config.ts:32/51` - dev-server fixture root and `server.fs.allow` are pinned
  to `path.resolve(here, "..")` (the app dir), inside the root.

## Declared External References

These are documentary references recorded per the audit, not runtime data the
project loads. Neither is moved into the repo (PII and licensing reasons), and
neither is needed to build, test, or run this project.

- **The predecessor repo** - read-only reference on the development machine;
  PII exclusion rules in CLAUDE.md govern every interaction with it. Concepts
  and isolated frontend techniques were ported; code and data were not.
- **TSDF (TieredSovereignDataFramework)** - the standard this project's
  provenance and tiering model aligns to; referenced conceptually in
  DECISIONS.md.

## Self-Containment Verdict

**SELF-CONTAINED.**

The project builds and runs with no file outside its root. Rust inputs live
under `core/`, TypeScript inputs under `app/`, synthetic data under
`fixtures/`, and schemas under `schemas/`. External code dependencies are
pulled from crates.io and the npm registry via in-project manifests
(`core/Cargo.toml` workspace + per-crate `Cargo.toml`, `app/package.json` +
`app/package-lock.json`), not from sibling directories. The only external
references found are documentary: sibling repos named in prose, and
user-profile tooling for the optional Codex CLI (`$CODEX_HOME`). None is
consumed by a build or at runtime, and none is broken.

What keeps it from being fully hermetic (does not lower the class, but worth
noting):

- The app depends on the gitignored wasm-pack output
  `core/crates/cn-wasm/pkg/`. A restore from tracked files alone must run
  `wasm-pack build` before the app builds. This is in-project and rebuildable,
  so it does not affect the verdict.
- History durability: until the first push to the conditionally opened public
  remote (preconditions above), git history exists only locally. Mitigation is
  machine-local operational practice, recorded outside the repo (`_private/`,
  gitignored). The public remote, once pushed, holds code only - it is not a
  backup answer for operational or pilot data (G-BACKUP / D-026 remains
  ACCEPTED, per HANDOFF.md).

## Runtime Notes

Out of audit scope, recorded so a restore host can be provisioned (versions per
docs/ENVIRONMENT.md, 2026-07-06): git 2.55, Node 24.14.1 / npm 11.12.1, rustup
1.29.0 with rustc/cargo 1.96.1 (Rust edition 2024), wasm-pack 0.15.0, and
PowerShell 7 as the shell for the scripts and hooks. Vite is pinned to 7.x for
vite-plugin-singlefile compatibility. The Codex CLI (0.142.5) is an optional
offload engine and is not required to build, test, or run the project. These
are installed toolchains, not files inside the project.

## Verification (adversarial pass)

Second reviewer, 2026-07-11, read-only except this file. Goal: refute or extend
the SELF-CONTAINED verdict from search angles the first pass did not lead with.
Result: verdict CONFIRMED. Two documentary references were added for
completeness; neither changes the class. Machine-specific verification detail
(existence re-checks on absolute paths, exact build-artifact measurements)
lives in the machine-local supplement (`_private/`, gitignored).

### Corrected / added (audit misses, all documentary - verdict unchanged)

- **Four named sibling integration targets.** The first pass folded a
  DECISIONS.md line into a generic workspace-parent IGNORE row. That line,
  plus `docs/design/integration-plan-2026-07-06.md`, `docs/NEXT_SESSION.md:20`,
  `docs/PROJECT_PLAN.md:193-194`, `docs/research/graph-networks-report-2026-07-06.md`,
  and HANDOFF.md, name **cap-assessor, TCR-policy-scanner, GeoBase,
  engagement-database** as future integration sources. All four exist. They are
  explicitly post-pilot ("integrate after the pilot"), so nothing is consumed
  by the current build or runtime - the same documentary tier at which the
  audit chose to DECLARE TSDF. Added as their own Findings row.
- **External GitHub repo URL.** `docs/THE_STORY.md:105` cites
  `github.com/atniclimate/pnw-tribal-dashboard and related repos`. The audit's
  URL row covered only the LICENSE URLs. Added; documentary, not a dependency.

### Confirmed (independently re-verified)

- **Verdict + no broken refs.** Existence re-checks on every declared external
  reference and on `core/crates/cn-wasm/pkg/cn_wasm.js` all passed.
- **npm graph has no local/file deps.** `app/package-lock.json` has 149
  `resolved` entries, every one `https://`; zero `file:` / `link:` / `git+` /
  `portal:`.
- **Cargo graph is in-tree.** Every `path =` dep is a sibling crate (`../cn-*`)
  inside `core/`; no `[patch]`, `[replace]`, `git =`, `registry =`, or
  `.cargo/config`.
- **Compile-time embeds and file I/O stay in-root.** `include_str!` targets
  resolve to `fixtures/templates/*` by path arithmetic; `cn-store`
  `fs::read`/`fs::write` take a runtime path argument (no hardcoded external
  path); tests use `std::env::temp_dir()`.
- **TS/JS path resolution stays in-root.** Every `path.resolve(here, "../..")`,
  `fileURLToPath(new URL("../../..", ...))`, and `vite.config.ts`
  `server.fs.allow` resolves to the repo root or below. Runtime `fetch()` hits
  only same-origin `/fixtures/...`.
- **Scripts/hooks carry no absolute paths.** `install-hooks.ps1`,
  `scripts/hooks/pre-commit`, and `pii-scan.ps1` derive the root from
  `git rev-parse --show-toplevel`; `core.hooksPath` is the relative
  `scripts/hooks`.
- **Git had no external tether at audit time (2026-07-11).** `.git/config` had
  no remotes (D-026 as then in force), no `objects/info/alternates`, and no
  worktrees - no borrowed object store or linked working tree pointing outside
  the root. See the Summary for the 2026-07-24 conditional remote opening
  (D-053/D-055).
- **No hidden config surfaces.** No `.github/`/CI YAML, no `.vscode/`, no
  Makefile, `.bat`/`.cmd`/`.sh`, or `.editorconfig`; no `.env*`; no
  symlinks/junctions; no UNC (`\\server`) paths; no cloud-drive or
  `/mnt`-style mounts; no path-shaped values inside the JSON/JSONL fixtures or
  schemas. Cron mentions are Claude Code scheduled-agent wake hooks, not
  filesystem references.
