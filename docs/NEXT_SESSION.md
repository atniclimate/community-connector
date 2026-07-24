# NEXT_SESSION.md - Resume Brief and Remaining Questions

> For the next session with the human present. A resuming director: read
> CLAUDE.md -> HANDOFF.md -> this file. Refresh the Brief at each session
> end (CLAUDE.md session-end protocol).

## The 60-second brief (refreshed 2026-07-24, end of the gate-grill + true-up session)

All six queued human gates were answered and ratified (D-050..D-055): the
convention is 2026-09-14 with internal pilots (trusted groups) in August;
v0.1.0 is ratified as the convention-arc finish line with a real-usage
acceptance bar (~150 expected / 300 max signups); intake is in-app facilitator
entry plus a QR -> GitHub Pages static form -> sealed-envelope -> Cloudflare
relay path, everything landing in a facilitator pending-review queue; ATNI
authors the vocabulary post-stability; the license is PolyForm Noncommercial
1.0.0 (already tracked); the three root docs get tracked after a pre-publish
sweep. A two-agent look-back (D-056) then reconciled the docs, required ADR-005
for the intake path, resequenced the route (snapshot pipeline moves after the
intake pipeline; P1.3 and most of Phase 4 defer), and wrote the August risk
register (facilitator key custody, queue durability before relay-wipe, the
sweep on the pilot critical path, submission dedup, human lead times).
check-all was re-verified 11/11 green. The public remote
`atniclimate/community-connector` exists (created by the human, empty);
NOTHING pushes until license + sweep + stability. Execution resumes at HANDOFF
next-action 1 (R2 EntityDetail fixes, D-049) with the long-lead gate-openers
(next-action 2) in parallel. The human intends ultracode-scale orchestration
for the build-out.

## How to resume work (for the director)

1. Read HANDOFF.md - state, ordered next actions, non-negotiables.
2. If the human is present: the two highest-value asks are scheduling the
   recorded ATNI Climate collective checkpoint (it must precede the FIRST
   August ingestion, D-050) and turnaround expectations for the D-023 review
   of the intake-form/consent text once drafted.
3. If autonomous: verify Codex health, then run next-actions 1 and 2 in
   parallel (R2 fixes + fresh adversarial round; D-055 sweep, ADR-005 draft,
   form-text draft, keygen ceremony design). Prefer small bounded Codex jobs.
4. Re-arm the 8:00 AM safety cron if the usage-failover directive stands.

## Remaining open questions (defaults keep autonomous work unblocked)

**Q-CHK. Collective checkpoint scheduling.** When can ATNI Climate record its
approval of the network activity? It must precede the first internal-pilot
ingestion of real people (D-030/D-050). Default: engineering proceeds on
synthetic data; no real ingestion until the recording exists.

**Q-TEXT. D-023 review loop.** Who reviews the intake-form and consent wording
besides you (ATNI Climate?), and how fast can it turn around? Default: the
draft lands in docs/design/ marked DRAFT; nothing goes live.

**Q-STABLE. Define "stable enough to push/deploy".** Proposed definition: R2
fixes landed + fresh adversarial round clean + check-all green + ADR-005
accepted. Confirm or tighten in one line. Default: that definition.

**Q-D. Part 7 process items (defaults per D-019):** autonomy = full with
ARCHITECTURE-redesign parking; spend = approved for the Cloudflare relay only;
cadence = 8:00 AM safety cron + usage failover stand. Correct in one line.

**Q-E. Part 8 retrospective (open discussion, never blocking):** what in the
output so far misses the mark; predecessor-demo reception lessons; anyone else
who should see the plan (committee members, ATNI staff).

**Q-F. Aesthetic check (deferred from Q5.4, D-038).** Runs inside the DESIGN
sitting: does Hearthlight feel right; cultural considerations for palette and
shape language. Default: Hearthlight stands, refined by evidence. The human
also plans a Claude-Design pass on front-facing text at the later stage
(D-051).

## Decisions already made that the human may want to revisit

D-008 codex sandbox bypass; D-026 backup risk accepted (re-raised every
decision session by standing rule - note the public remote will hold code
only, never data, so it is not the backup answer); D-032 TSDF codes primary in
the UI; D-037 in-app story authoring in v0.1; the v0 layout being client-side
deterministic; D-056.2 ownership-at-approval (approved remote submissions land
unowned, facilitator-created; owner-binding them to submitters later is an
authority-matrix change plus an adversarial round). Revisiting any is a
DECISIONS entry, not a rewrite - say the word.
