# AGENTS.md - Invariant Checklist

Canonical review standard for every diff in this repo, whether authored by the
director, Codex, or a human. A violation of I1-I4 or I6 is a blocking finding.
Docs use hyphens, never em dashes.

## Invariants

- **I1.** No PII in the repo, in fixtures, in tests, or in any Codex prompt.
  Synthetic names/emails only, from an obviously-fake namespace
  (`@example.test`). The pre-commit PII scan must pass on every commit.
- **I2.** Permission checks live only in `cn-perm`. Any permission logic
  found in the app layer is a blocking review finding.
- **I3.** No silent error swallowing. No `catch {}` / `let _ =` on fallible
  provenance, validation, or IO paths. Errors are typed, propagated, and
  surfaced in the validation report.
- **I4.** App state mutates through the explicit state machine in
  `app/src/state/` only. Direct mutation elsewhere is a blocking finding.
- **I5.** Functions over ~60 lines or modules over ~500 lines need a
  documented reason or a split.
- **I6.** Every entity and edge carries a provenance envelope and a tier;
  constructors that skip them do not exist.
- **I7.** Every persisted format (group templates, op log, exports, stories)
  carries an explicit schema version; readers reject unknown majors loudly.
- **I8.** Builds are deterministic; the single-file snapshot stays under 5MB;
  size is checked in the build script, not by eyeball.
- **I9.** Accessibility baseline: interactive elements are keyboard-reachable
  and ARIA-labeled; animation respects `prefers-reduced-motion`; layouts
  hold at 375px width.
- **I10.** Docs use hyphens, never em dashes.
- **I11.** Conventional commits; Codex session ids in commit messages for
  offloaded work; every ladder climb past rung 1 has a `DECISIONS.md` entry.
- **I12.** Intermediate validation between pipeline/ingest stages, with a
  machine-readable validation report; warnings are visible, never dropped.

## Repo commands

```
# Rust core (run from core/)
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
wasm-pack build crates/cn-wasm --target web

# App (run from app/)
npm run typecheck        # tsc --noEmit
npm run build            # vite build
npm run build:snapshot   # single-file offline build + size budget check

# Cross-cutting (repo root)
pwsh scripts/pii-scan.ps1          # PII tripwire, also wired as pre-commit hook
```

## Red data (never read, never include in prompts)

All real partner PII; the exclusion list in CLAUDE.md "Predecessor repo rules";
any credentials or tokens; any future real-community data until the owner clears
it in writing. Nothing is currently cleared. Green by default: this repo's code,
diffs, design docs, synthetic fixtures, logs from synthetic runs.
