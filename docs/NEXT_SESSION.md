# NEXT_SESSION.md - Launch cards (refreshed 2026-09-12, pickup + discovery close)

> For the next session. Reading order: CLAUDE.md -> HANDOFF.md (its "Pickup 2026-09-12"
> block first) -> this file. Session plan of record: `SESSION_ROSTER.yaml` (root).
> Evidence: `TRACE.yaml`, `docs/planning/RECONCILIATION-2026-09-12.md`,
> `docs/research/discovery-2026-09-12.md`. Previous briefs live in git history of this
> file (last one 2026-08-11).

## The 60-second brief

The remote-intake relay is COMPLETE and accepted (D-089). Nothing has been pushed since
2026-08-11; local `main` is 31 commits ahead of origin plus this session's four docs
commits. check-all is 11 of 12: rust-clippy fails on one new 1.98 lint at
`core/cli/src/intake/keymat.rs:258` (toolchain drift, no `rust-toolchain.toml`) - the
first session below fixes it. The ATNI convention is 2026-09-14 and the human ruled that
the mainstage reveal's visuals come first; the deploy bar (D-059.8) stays UNMET and
nothing goes live for the convention. Roster order: R0, R2, R4a, R4b, R7 before the
convention; R1, R3, R5, R6, then the maintenance chain after. Standing rulings no
session may "fix": D-032, D-037, D-051, D-056.4, ADR-004.

## Launch card: S-R0 - toolchain pin + clippy fix + troika dependency fix (NEXT)

**Tier** T1 - **Budget** ~12k tokens - **Gate** `pwsh scripts/check-all.ps1` (12/12)

```powershell
# PowerShell - launch through the `claude` wrapper only (C:\dev\CLAUDE.md Rule 9)
Set-Location I:\community-connector
claude --model claude-sonnet-5 --effort medium
```

Opening prompt (paste as the first message):

> Role: fixer. Three small mechanical fixes, each its own conventional commit, check-all
> green before each commit.
> Outcome: `pwsh scripts/check-all.ps1` prints 12 of 12 PASS on this machine.
> Scope fence: `core/cli/src/intake/keymat.rs` (line 258: remove the redundant `&` in
> the `format!` argument), a new `rust-toolchain.toml` at the repo root pinned to the
> version check-all is green on (cargo/rustc 1.98.1 today), and `app/package.json`
> (move `troika-three-text` from devDependencies to dependencies; `app/src/viz/labels.ts:2`
> imports it). Touch nothing else. Do not change the pinned versions of anything else.
> Read first: HANDOFF.md "Pickup 2026-09-12" block; SESSION_ROSTER.yaml entry S-R0;
> `core/cli/src/intake/keymat.rs` lines 245-262; `app/package.json` whole file.
> Verification: run `pwsh scripts/check-all.ps1` and paste the summary table before any
> completion claim. The pre-commit hook must fire (`git config core.hooksPath` must
> resolve to `I:\community-connector\scripts\hooks`; run `scripts/install-hooks.ps1` if not).
> Stop when: check-all is 12/12 and the three commits exist. Mark S-R0 done in
> SESSION_ROSTER.yaml with a one-line outcome.

Then continue in the same terminal with S-R2 (below) or hand off.

## Launch card: S-R2 - ATNI convention template, 15 committees, synthetic fixture

**Tier** T3 - **Budget** ~60k tokens - **Gate** `pwsh scripts/check-all.ps1` incl.
`npm run validate:templates`, plus pii-scan clean and every value under `@example.test`

```powershell
Set-Location I:\community-connector
claude --model claude-sonnet-5 --effort high --permission-mode plan
```

> Role: template author (plan first, then code).
> Outcome: a new synthetic group template + ops fixture under `fixtures/` for the ATNI
> convention pilot, validated by check-all: a `committee` kind with exactly these 15
> instances - Energy; Taxation; Education (K-12); ICWA; Law & Justice; Philanthropy;
> Telecomms & Tech; Food Sovereignty; Economic Development; Native Vote; TERO; Gaming;
> Drug Abuse & Prevention; Housing; Climate Resilience - a `member_of` edge kind, a
> `connected_to` edge kind, and person attributes per discovery memo D-094c. Add the
> alias-table block to the group-template schema as an additive optional field with a
> version bump (shape only; no matching logic - that is R3).
> Scope fence: `schemas/group-template.schema.json`, `fixtures/templates/`,
> `fixtures/groups/`, `app/scripts/` validators if the schema bump needs them. No app UI,
> no cn-ingest, no cn-perm. All entries T1 (D-034). Synthetic data only.
> Read first: SESSION_ROSTER.yaml entry S-R2; discovery memo Track B and D-094c;
> `fixtures/templates/research-network.template.json`; the schema file whole.
> Verification: check-all summary pasted; `pwsh scripts/pii-scan.ps1` pasted.
> Stop when: check-all 12/12, the new template validates, and S-R2 is marked done.

## Launch card: S-R1 - deploy-bar clearance (AFTER the convention)

**Tier** T4 - **Budget** ~150k tokens - **Gate** check-all 12/12 AND the new Playwright
smoke passes against `wrangler dev` AND `npm test` green in both `relay/` and `form/`

```powershell
Set-Location I:\community-connector
claude --model claude-sonnet-5 --effort high --permission-mode plan
```

> Role: deploy engineer (plan first). Outcome: the engineering half of D-059.8 is closed
> with pasted evidence: (1) a real-browser Playwright smoke of the BUILT form under its
> real CSP (seal a synthetic submission, POST to a local `wrangler dev`, assert a
> receipt); (2) `.github/workflows/` GitHub Pages deploy workflow (configure-pages@v5,
> upload-pages-artifact@v4, deploy-pages@v4; `pages: write`, `id-token: write`;
> deploys exactly the manifest's file set, never the manifest) with a post-deploy D8
> fetch-and-hash step against the LOCAL pin; (3) the R4-5 build guard that fails when
> `CN_FORM_RELAY_ORIGIN` is the localhost default; (4) a keygen ceremony REHEARSAL on
> synthetic keys via `cn intake selftest --dry` and the ceremony checklist.
> Scope fence: `form/`, `relay/` tests, `scripts/`, `.github/workflows/` (guarded: ask
> before writing there), `docs/runbooks/`. Never deploy, never run the real ceremony,
> never touch `_private/`.
> Human steps you hand back as a checklist, never perform: D-023 sign-off; the real
> ceremony; Pages source set to "GitHub Actions"; the Workers tier (D-092c).
> Read first: SESSION_ROSTER.yaml entry S-R1 (reads with line ranges); discovery memo
> Track A; `docs/runbooks/intake-relay-deploy.md` (DRAFT, do not execute).
> Verification: every gate leg pasted; a claim-verifier subagent re-runs the smoke and
> both vitest suites before any "green" claim.
> Stop when: all four gate legs pass and the human checklist is written.

## Mid-session rules (all cards)

- Skipped a file or test: raise effort one notch, stay here.
- Wrong with full context: `/clear`, relaunch on the next model up with this card plus
  one line naming the missed fact.
- Two failed corrections on one bug: `/clear`. Context past ~130k: write the handover
  into HANDOFF.md "next actions", `/clear`, split the task.

## Questions for the human (one-line answers unblock the roster)

1. Record, amend, or reject D-090c..D-096c (`docs/research/discovery-2026-09-12.md`).
2. Which Claude plan or seat is this? (`SESSION_ROSTER.yaml` `plan_tier` is unverified.)
3. D-023: will the consent-text sign-off happen before 2026-09-14? If not, the reveal
   is demo-only on synthetic data (D-091c).
