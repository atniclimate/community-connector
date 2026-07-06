# NEXT_SESSION.md - Resume Brief and Human Interview

> For the next session with the human present. A resuming director: read
> CLAUDE.md -> HANDOFF.md -> this file. Give the human the Brief verbatim-ish,
> then offer the Interview (they choose how much to answer; every question
> has a default so nothing blocks). Refresh the Brief at each session end
> (CLAUDE.md session-end protocol).

## The 60-second brief (refreshed 2026-07-06, end of session 1)

In one overnight-and-morning session, Community Navigator went from an
empty folder to: a complete, adversarially-reviewed Rust/WASM core
(permissions, event log, graph queries - with a property test proving no
projection ever leaks above a viewer's access), an evidence-based renderer
decision measured on this machine's GPU, and a working app skeleton that
loads a synthetic community of 120 people and places and draws it as an
instanced 3D constellation (first screenshot:
docs/design/screenshots/base-renderer-2026-07-06.png - node colors need a
fix, that is queued). ~57 commits, every one verified and PII-scanned.
Roughly half of Phase 3 (frontend) remains, then ingestion (Phase 4),
personal mode (Phase 5), and hardening to v0.1.0 - an estimated 10-13
focused sessions, mapped in docs/PROJECT_PLAN.md.

## How to resume work (for the director)

1. Read HANDOFF.md - it has the exact in-flight state and next actions.
2. If the human is present: deliver the Brief, then the Interview below.
3. If autonomous: continue PROJECT_PLAN session sequence from HANDOFF.
4. Re-arm the 8:00 AM safety cron if the usage-failover directive stands.

## The Interview

Ordered by leverage. Each: why it matters, options, and the default if
unanswered (defaults keep autonomous sessions unblocked; answers convert
to DECISIONS.md entries or ADR inputs).

**Q1. Backup / remote (HIGHEST PRIORITY - risk, not feature).**
The repo exists only on this laptop; a disk failure erases everything.
Creating any remote is a human gate. Options: (a) private GitHub/GitLab
remote; (b) a second local drive / network share via `git bundle`; (c)
accept the risk for now. Default if unanswered: (b)-style local bundle
backups CANNOT be created autonomously either (still leaves the machine?
no - same machine is pointless; a share is a remote). So the true default
is (c) with this risk re-raised every session. One line answers it.

**Q2. License variant (standing).**
"polyform" was chosen; the director selected PolyForm Noncommercial 1.0.0.
Confirm, or name Internal Use / Small Business. Default: Noncommercial
stands.

**Q3. Identity for personal mode (gates Session D / Phase 5).**
When an individual manages their own record and trust grants, how do they
prove who they are on a local-first app? Options: (a) device-local
credential (passkey/OS keychain) per person; (b) committee-issued claim
codes (governance hands a person a one-time code that binds their record);
(c) defer personal mode past v0.1.0 (viewer switcher stays a dev tool).
Also: should choosing any identity STANDARD (DID, OAuth, passkeys) wait
for the network decision? Default: design ADR-005/D assuming (b)-style
local claim codes with no external standard commitment - reversible and
sovereignty-aligned - and personal mode ships behind it.

**Q4. First real community + data (gates Session E scheduling).**
Which dataset goes first - CPF-RCN (migration recipe) or a fresh ATNI
committee entering data by hand? Who is the governance authority that
assigns tiers for it, and what does the consent/FPIC checkpoint look like
in practice? Default: recipe gets WRITTEN (docs only) but nothing real is
ingested; fixtures remain the only data.

**Q5. Sovereignty language in the UI (feeds Sessions E and F).**
How should tiers and sharing be worded for community members - reuse
TSDF/CARE vocabulary, ATNI's own terms, or plain-language equivalents
("kept by the community", "shared with trusted partners")? Does ATNI have
existing data-governance protocol documents the UI should mirror? Default:
plain-language equivalents with TSDF tier codes shown secondarily.

**Q6. Accessibility reality (feeds Session A).**
Known assistive-technology users among target communities? Elder-focused
needs beyond WCAG (larger type, simpler modes)? Any language localization
on the v0.x horizon? Default: WCAG 2.2 AA mechanics per the design brief,
English only, font-scale token ready but no UI for it yet.

**Q7. Group creation reality (feeds Session B).**
Who actually sets up a group - a facilitator with a laptop in a meeting, or
individuals on their own? Should template AUTHORING (new community types,
not just new groups) be in-app for v0.1 or remain a JSON-file task?
Default: facilitator-led wizard; template authoring stays JSON.

**Q8. Duplicate handling (feeds Session C).**
When ingest finds two records that look like the same person, should it
ever auto-merge, or always queue for human review? Default: always queue;
auto-merge only on exact-key matches (same email), and even then reversibly.

**Q9. Demo target (prioritization).**
Is there a date/venue for showing this (like the predecessor's demo)? Is
the offline single-file snapshot the primary vehicle again, or the live
app? Default: snapshot-first polish, matching the predecessor's proven
distribution.

**Q10. Improvements retrospective (discussion, not decision).**
- The autonomous overnight run: right level of autonomy, or should big
  judgment calls (e.g. REDESIGN verdicts) have waited for you?
- Codex spend felt acceptable? (~20 offloaded tasks this session.)
- Anything about the direction - Hearthlight aesthetic, the fisheries
  fixture's framing, the plan's ordering - you want steered differently
  before more gets built on top?
- Session cadence preference going forward: overnight autonomous runs,
  supervised working sessions, or decision-session-then-autonomous bursts?

## Decisions already made that the human may want to revisit

D-008 codex sandbox bypass; D-011 license variant selection; D-016 cn-sync
deferred to Phase 5; ADR-004 renderer (evidence attached); the v0 layout
being client-side deterministic (base-renderer blueprint). Revisiting any
is a DECISIONS entry, not a rewrite - say the word.
