# Pilot Field-Validation Evidence - Template (P5.10)

Status: TEMPLATE - contains placeholders only, no filled data. This file defines
the privacy-safe evidence artifact for the convention pilot. It is the
field-validation gate input consumed by P9.6 ahead of 1.0 ratification.

## How to use this template

- Copy this file to create a filled instance per pilot or rehearsal run. A filled
  instance containing any real-community content lives OUTSIDE this repository
  (or in gitignored staging) per the real-data gate (D-030, D-034). Only fully
  aggregate, non-identifying instances may ever be committed, and only after the
  Section 5 sign-off confirms that property.
- Absolute rules for any filled instance:
  - No participant data. No names, emails, phone numbers, affiliations, free-text
    quotes attributable to a person, or any pairing that identifies an individual.
  - Field observations are aggregate-only (Section 3). Suppress any count where
    fewer than 5 individuals contributed, recording it as `<5`.
  - Issue summaries are paraphrased and consented (Section 4). Consent records
    themselves stay outside the repository; reference them by id only.
  - Synthetic references only: fixture names, commit hashes, check outputs.
    The two repo fixture domains are `research-network` and `fisheries-committee`.
  - Any community-facing wording added while filling this template requires human
    review before use (D-023).

---

## 1. Dated rehearsal checklist (P5.9 acceptance record)

Rehearsal date: `[YYYY-MM-DD]`
Operator role: `[facilitator | maintainer]` (role only in committed copies)
Repo commit exercised: `[short-hash]`
Machine/browser class: `[e.g. laptop, offline snapshot, browser family]`

| Step | Expected result | Observed (pass/fail) | Notes (non-identifying) |
| --- | --- | --- | --- |
| Ingest synthetic fixture batch | `[expected]` | `[pass/fail]` | `[notes]` |
| Reveal permission-filtered projection | `[expected]` | `[pass/fail]` | `[notes]` |
| Route a need-term to a pathway | `[expected]` | `[pass/fail]` | `[notes]` |
| Story walkthrough | `[expected]` | `[pass/fail]` | `[notes]` |
| Re-ingest (idempotency) | `[expected]` | `[pass/fail]` | `[notes]` |
| Offline snapshot open and use | `[expected]` | `[pass/fail]` | `[notes]` |
| Keyboard-only pass of the arc | `[expected]` | `[pass/fail]` | `[notes]` |

Add rows as the rehearsed arc grows. A row is `pass` only against the written
expected result, not a hoped-for outcome.

## 2. Synthetic regression bundle description

Describes the reproducible synthetic evidence that the exercised build behaves
as claimed. No real data appears in this section by construction.

- Repo commit: `[short-hash]`
- Fixture inputs: `[fixture file paths, e.g. fixtures/groups/*.ops.jsonl]`
- Checks run and results: `[check name -> pass/fail, one line each]`
- Snapshot size at build: `[bytes]` (budget: under 5MB, I8)
- Artifacts produced: `[e.g. snapshot file name, validation report path]`
- Deviations from a clean run: `[none | list, with issue ids from Section 4]`

## 3. Aggregate field observations (non-identifying format)

Each observation is a counted or measured aggregate. No row may describe a
specific person, and no combination of rows may isolate one.

| Obs id | Category | Metric | Value | Collection method | Suppression applied |
| --- | --- | --- | --- | --- | --- |
| `OBS-001` | `[e.g. intake, navigation, performance]` | `[what was counted/measured]` | `[n or <5]` | `[e.g. facilitator tally, timer]` | `[yes/no]` |

Allowed metric examples: number of intake submissions, validation failures by
category, task completion counts, task durations, device/browser class counts.
Disallowed: free-text responses, per-person timelines, any metric with n of 1
attributable to a known individual.

## 4. Consented issue summaries

One entry per issue a community member raised and consented to have recorded.
Paraphrase only; no verbatim quotes; no identifying details. The consent record
itself lives outside the repository with the pilot materials.

| Issue id | Date | Paraphrased summary (non-identifying) | Consent ref (external) | Severity | Disposition |
| --- | --- | --- | --- | --- | --- |
| `ISS-001` | `[YYYY-MM-DD]` | `[what was reported, in the recorder's words]` | `[external record id]` | `[low/med/high]` | `[open/fixed/wontfix/parked]` |

Suggested wording when asking permission to record an issue:

> DRAFT - PENDING HUMAN REVIEW (D-023): "May I write down a short summary of
> this issue, without your name or any identifying details, to help improve the
> tool?"

Do not use that wording with anyone until it has passed human review.

## 5. Human sign-off record

A machine cannot close this section. The signer attests, after reading the
filled artifact end to end, that:

1. Sections 1-4 are complete for the run being signed.
2. The artifact contains no PII and no participant-identifying content.
3. Aggregate suppression (Section 3) was applied wherever required.
4. Every Section 4 entry has a matching external consent record.
5. Storage location of the filled artifact complies with the real-data gate
   (D-030, D-034) and the tier authority's decisions.

Signer: `[operator name - role]` (no contact info in committed copies)
Date: `[YYYY-MM-DD]`
Scope: `[rehearsal | pilot run id]`
Decision: `[approved as evidence | returned for correction]`
Notes: `[optional, non-identifying]`

---

Consumed by: P9.6 (field-validation gate). Related: P5.9 (rehearsal), D-023
(community-facing text review), D-030/D-034 (real-data gate), AGENTS.md I1.
