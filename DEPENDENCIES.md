# community-connector - External Dependency Audit

(audit note 2026-07-24: this is the world-readable revision of the 2026-07-11
read-only audit, produced by the D-055 pre-publish sweep. Machine-local
operational detail - backup topology, mirror and restore procedure, absolute
local paths, and exact machine measurements - was split out of this document
and lives outside the tracked tree in `_private/`, gitignored. This doc is
subordinate to HANDOFF.md and DECISIONS.md. Docs use hyphens, never em dashes,
per CLAUDE.md and AGENTS.md I10.)

(true-up note 2026-08-11: re-verified after the remote-intake relay work
landed, blueprint steps 1-11. That work added three in-project npm project
roots (`form/`, `relay/`, `scripts/`) and new crates.io/npm registry packages;
it added no new reference to any file or directory outside the project root.
The self-containment verdict is UNCHANGED. The new registry packages fall
OUTSIDE this audit's documented scope - it tracks external-path references,
not the package inventory - so they are NOT catalogued here; see the Scope
note below for where the authoritative dependency list lives.)

Scope: every reference in the repo to a file or directory outside the project
root, verified and classified. Installed programs and toolchains are out of
scope except the single Runtime Notes paragraph. Fixtures are synthetic; no
file contents are quoted beyond what path classification needs.

Scope note - registry packages are OUT of scope here. This is a path-reference
audit, not a package inventory. Crates.io crates and npm packages are fetched
by a package manager from in-project manifests; they are not references to a
file or directory outside the root, so they are not enumerated in this
document. The authoritative per-dependency impact list for the remote-intake
relay work (new crates and npm packages with purpose) is
`docs/blueprints/intake-relay.md` section 9 "Dependency impact"; the in-tree
manifests (`core/cli/Cargo.toml`, `core/crates/cn-ingest/Cargo.toml`,
`form/package.json`, `relay/package.json`, `scripts/package.json`, each with a
lockfile) are the ground truth. Licenses of third-party assets that SHIP to
users live in `docs/NOTICE-third-party.md`. What this audit does track for that
work is below: it added no external-path reference, and the four in-project npm
project roots and the Cargo workspace remain registry-only.

## Summary

- No reparse points (junctions/symlinks) and no `.lnk` shortcuts anywhere in
  the tree.
- No `.env` / `.env.*` files present; no secrets to redact.
- Remotes: at the original 2026-07-11 audit the repository had no git remotes
  (D-026, an accepted single-machine risk). On 2026-07-24 the human
  conditionally opened the remote/publishing gate for exactly one path - the
  public remote `atniclimate/community-connector` - and the first push was
  executed the same day (D-060). As of the 2026-08-11 true-up the single remote
  `origin` = `https://github.com/atniclimate/community-connector.git` is
  configured and the repo is public and continuously pushable; there are still
  no other remotes, no `objects/info/alternates`, and a single worktree at the
  root (no borrowed object store, no linked working tree outside the root). The
  public remote holds code only - it is not a backup for operational or pilot
  data (G-BACKUP / D-026 remains ACCEPTED). All other gates stand unchanged.
- Every candidate external path was verified at audit time: all exist. Zero
  broken references.
- All build and runtime inputs are either in-project or fetched by a package
  manager (crates.io, npm registry). No FILE outside the root is consumed by a
  build or at runtime. The only external-path references are documentary
  (sibling repos named in prose). The build/test inputs now span four
  in-project npm project roots (`app/`, `form/`, `relay/`, `scripts/`) plus the
  Cargo workspace; all remain registry-only (verified below).
- New runtime-egress posture (does not add an external-path reference). The
  remote-intake relay work introduces OUTBOUND network to external origins: the
  native `cn intake pull` puller (CLI binary only) fetches from the Worker relay
  and Pages origins, and the deployed `form/`/`relay/` talk to those origins.
  This is not a filesystem reference and does not lower the self-containment
  class: it is inert until the D-059.8 deploy bar clears, the origins are
  supplied by an OFF-REPO puller config (no external origin is hardcoded in
  tracked source - verified), and ADR-005 D1's module fence keeps the sole HTTP
  client (`ureq`) in the `cn` CLI binary, never in any `cn-*` core crate or the
  app. The app runtime still fetches only same-origin `/fixtures/...`.
- The app imports the wasm-pack output `core/crates/cn-wasm/pkg/cn_wasm.js`,
  which is in-project but gitignored build output. It is not an external
  dependency; it is a restore-time rebuild step
  (`wasm-pack build crates/cn-wasm --target web`). Likewise `form/dist/` and the
  three new `node_modules/` (`form/`, `relay/`, `scripts/`) are gitignored,
  rebuildable, in-project build artifacts, not external dependencies; the D8
  deploy manifest `form/dist.manifest.json` IS tracked as a provenance record.

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
under `core/`, TypeScript inputs under `app/` and now also `form/` (the static
Pages form) and `relay/` (the Cloudflare Worker), synthetic data under
`fixtures/`, and schemas under `schemas/`. External code dependencies are
pulled from crates.io and the npm registry via in-project manifests - the
`core/Cargo.toml` workspace + per-crate `Cargo.toml`, and now four npm project
roots each with its own lockfile: `app/package.json`, `form/package.json`,
`relay/package.json`, and the manual-run `scripts/package.json` - not from
sibling directories. The only external references found are documentary:
sibling repos named in prose, and user-profile tooling for the optional Codex
CLI (`$CODEX_HOME`). None is consumed by a build or at runtime, and none is
broken.

Third-party runtime code vendored into the new surfaces (registry packages, in
scope only as a self-containment note, not enumerated as external-path
references): `form/` vendors `libsodium-wrappers` as its ONLY runtime
dependency (it seals submissions in-browser); `relay/` vendors no runtime npm
package (it uses the platform Web Crypto API; its npm deps are dev-only -
`wrangler`, `@cloudflare/vitest-pool-workers`, `@cloudflare/workers-types`);
and `scripts/` vendors `libsodium-wrappers` for the manual cross-implementation
crypto test-vector generator (not part of check-all). The full per-dependency
list with purpose is `docs/blueprints/intake-relay.md` section 9; shipped-asset
licenses are `docs/NOTICE-third-party.md`.

What keeps it from being fully hermetic (does not lower the class, but worth
noting):

- The app depends on the gitignored wasm-pack output
  `core/crates/cn-wasm/pkg/`. A restore from tracked files alone must run
  `wasm-pack build` before the app builds. This is in-project and rebuildable,
  so it does not affect the verdict.
- A restore must also run `npm install` (or `npm ci`) in each of the four npm
  roots (`app/`, `form/`, `relay/`, `scripts/`) to repopulate the gitignored
  `node_modules/`, and `form/`'s build regenerates the gitignored `form/dist/`.
  All are in-project, rebuildable from tracked manifests and lockfiles, so none
  affects the verdict.
- History durability: git history is now mirrored to the public remote
  `origin` (first push D-060, 2026-07-24); before that it existed only locally.
  The public remote holds code only - it is not a backup answer for operational
  or pilot data (G-BACKUP / D-026 remains ACCEPTED, per HANDOFF.md). Any
  machine-local mirror remains operational practice recorded outside the repo
  (`_private/`, gitignored).

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
  filesystem references. (Superseded 2026-08-11 for `.sh`: see the true-up
  below - in-project `.sh`/`.mjs` scripts now exist and carry no external
  tether.)

### True-up re-verification (2026-08-11, on-disk, remote-intake relay work)

Re-checked against the tree and manifests after blueprint steps 1-11 landed;
verdict CONFIRMED unchanged (SELF-CONTAINED). No new external-path reference was
introduced.

- **Three new npm lockfiles are registry-only.** `form/package-lock.json`,
  `relay/package-lock.json`, and `scripts/package-lock.json` have only
  `https://` `resolved` entries (97, 160, and 2 respectively); zero `file:` /
  `link:` / `git+` / `portal:`. Joins `app/`'s clean graph.
- **Cargo graph still in-tree.** The workspace gained crates.io registry
  packages (sealed-box/crypto, HTTP client, ceremony helpers - see
  `docs/blueprints/intake-relay.md` section 9), but still no `git =`,
  `[patch]`, `[replace]`, or `registry =` anywhere under `core/`; every `path =`
  dep is an in-`core/` sibling crate.
- **No external origin hardcoded in tracked source.** The relay/Pages origins
  the puller and form talk to come from an off-repo puller config; a scan of
  `form/src`, `relay/src`, and `core/cli/src` found no committed `workers.dev`
  or `github.io` origin (only placeholders like `<worker>` / `<org>`). ADR-005
  D1's module fence holds: the HTTP client lives only in the `cn` CLI crate.
- **New build artifacts are gitignored and rebuildable.** `form/dist/` and the
  `node_modules/` under `form/`, `relay/`, and `scripts/` are gitignored;
  `form/dist.manifest.json` (the D8 deploy manifest) is tracked as provenance.
- **In-project scripts, not tethers.** New `.sh`/`.mjs`/`.ps1` scripts under
  `scripts/` and `scripts/e2e/` (form build, crypto-vector generator,
  remote-intake e2e drivers) and the `core/cli/examples/` Rust example carry no
  absolute or external path; they resolve in-root. This supersedes the older
  "no `.sh`" observation above.
- **Git tether re-check.** One remote `origin` (the public
  `atniclimate/community-connector`, D-060); no `objects/info/alternates`;
  single worktree at the root. No borrowed object store, no linked working tree
  outside the root.
