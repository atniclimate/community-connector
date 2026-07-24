# CPF-RCN Migration Recipe (docs-only; P5.7 / D-031)

Status: recipe for the HUMAN to execute. This document was authored without reading
any red data. It never instructs an agent to read, copy, or transcribe predecessor
PII; every red-data-touching step is a human action. Docs use hyphens, never em
dashes.

## Purpose

The predecessor repo `C:\dev\CPF-RCN_demo` holds a real 3D network demo built with
real-partner PII. Community Navigator will not bulk-copy its code or data (CLAUDE.md
"Predecessor repo rules"). If, after the pilot and the real-data gate process
(CLAUDE.md; D-030/D-034), the maintainer wants to bring forward specific relationships
as consented pilot data, this recipe is the safe path: export a narrow slice, scrub it,
tier it, obtain FPIC, and ingest only the result - outside this repository.

## Hard fence (never crossed)

- The predecessor exclusion list is absolute (maintained machine-locally at
  `_private/PREDECESSOR-EXCLUSIONS.md`, gitignored, per D-059): no predecessor
  data directory, person file, or derived output, and no file pairing a real
  name with contact info. No agent (Claude or Codex) reads these, ever.
- No real PII enters this repository, any commit, any fixture, or any Codex prompt
  (Prime Directive 1, I1). Real pilot data lives outside the repo or in gitignored
  staging and is never committed.
- Migration is a human gate: it requires the recorded ATNI Climate collective
  checkpoint and per-record individual consent before any ingestion runs (CLAUDE.md
  real-data gate process).

## The sequence (human-executed)

1. **Scope (human, no export yet).** Decide the minimal set of entities/relationships
   worth migrating. Prefer relationships the community has already consented to share;
   default to excluding anything from the exclusion list. Write the scope down.
2. **Export a narrow slice (human).** From the predecessor repo, the maintainer exports
   only the scoped records into a working file on a gitignored staging path OUTSIDE
   this repo (e.g. `<external-staging-drive>\pilot-staging\` or a path in
   `.gitignore`). Agents do not run
   this export and do not read its output.
3. **Scrub (human).** In staging, remove or pseudonymize every direct identifier
   (names paired with contact info, emails, phone numbers, unreleased affiliations)
   down to what the consent instrument actually covers. Replace identities with the
   stable synthetic-safe references the pilot will use. Verify no exclusion-list file
   was copied.
4. **Tier (human, ATNI Climate authority).** Assign each entity and edge a TSDF
   sensitivity tier. Per D-034 the demo-wide default is T1 and the tier-assignment
   authority is ATNI Climate; do not lower a tier without that authority (tightening
   only; ADR export gates respect tiers, R10).
5. **FPIC (human, collective + individual).** Obtain the recorded ATNI Climate
   committee approval (collective checkpoint) and per-record individual consent (the
   intake-form consent instrument - in-app entry or the D-053 sealed-envelope QR
   relay, D-030). No record without both.
6. **Validate (agent-safe, synthetic-shaped only).** Once the staged slice is scrubbed
   and consented, it may be validated with `cn validate` / the ingest validator, which
   produces a machine-readable report (I12). If any value still looks like real PII,
   STOP - the scrub was incomplete; return to step 3. An agent may assist ONLY with a
   file the human affirms is fully scrubbed and consented.
7. **Ingest (once the importer exists; parks on G-RAT / P5.3).** The idempotent
   `cn-ingest` importer stamps `Origin::Ingested` + the assigned tier and a first-sight
   custody event; re-ingest is idempotent (D-030). Ingest reads only the scrubbed,
   consented staging file, never the predecessor repo.
8. **Verify + record (human).** Confirm the ingested graph in a permission-filtered
   projection; record the migration in the pilot evidence artifact
   (docs/pilot-evidence-template.md, P5.10) - aggregate/non-identifying only.

## What this recipe deliberately does not do

- It does not migrate the predecessor's code or renderer (port concepts only, never
  bulk-copy - CLAUDE.md).
- It does not automate any red-data-touching step. Steps 2-5 are human actions.
- It does not define importer semantics; those are the ratification-dependent
  `AtniIntakeBatchV0_1` work (integration plan 6.1, parks on G-RAT / P5.3).

## Cross-references

- CLAUDE.md: Human gates, Predecessor repo rules, real-data gate process.
- DECISIONS.md: D-030 (form-respondents-only + QR joins), D-031 (migration recipe),
  D-034 (T1 default + ATNI Climate tier authority).
- docs/pilot-evidence-template.md (P5.10): where a completed migration is recorded,
  PII-free.
