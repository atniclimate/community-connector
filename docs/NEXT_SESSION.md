# NEXT_SESSION.md - Resume Brief and Remaining Questions

> For the next session with the human present. A resuming director: read
> CLAUDE.md -> HANDOFF.md -> this file. Refresh the Brief at each session
> end (CLAUDE.md session-end protocol).

## The 60-second brief (refreshed 2026-07-24, end of the ADR-005 acceptance session)

**ADR-005 is ACCEPTED** (D-068) after EIGHT adversarial rounds in one day
(D-061..D-068; rounds 1-7 FAIL-and-amend with every finding verified
before judgment, round 8 pass). The remote-intake architecture is now
binding: native durable owner (`cn intake apply`; the browser app is
create-only), idempotent decision inbox (decision_generation CAS,
writeless replays, two-kind history), receipt-ledger reconciliation,
off-origin full-bundle pin, enforceable rotation cutoff, and consent
affirmation that survives the purge sweep. The P3.5/P3.6 director
blueprint (docs/blueprints/intake-pipeline.md) is written, aligned, and
carries the reviewer's implementation gates in its test lists. The
consent draft gained section 7: four wording conflicts for the D-023
pass - the removal-semantics question (no-longer-shown vs erasure) is a
human decision. Implementation then proceeded the same day: blueprint
steps 1-6 of 11 are LANDED and pushed (cn-store durable seam, cn-model
intake provenance block, cn-ingest queue formats + admission table +
recovery classification + near-dup + dedup + plan_approval, `cn intake
apply` - the native durable owner with the full decide -> apply ->
reload integration round trip - and the read-only cn-api/cn-wasm intake
facade with the no-leak extension proven on a real projection; check-all
green at every commit; D-069/D-070/D-071 record the deviations and
choices incl. the fixed queue file layout the app's FSA adapter will
write). Next action: steps 7-9, the app half (template->form renderer
P3.6, FSA create-only adapter, wizard panel P3.5), then tripwires and
fixtures, then the mandatory adversarial round on the whole
implementation diff (six commits, unreviewed).
An Asana refresh for this session is owed at the next session close.
The deploy bar (D-059.8) is unchanged - ADR-005 acceptance satisfied its
first condition; intake pipeline + keygen ceremony + D-023 remain.

## The previous brief (2026-07-24, end of the grill + first-push session)

**The repo is public.** The grill session resolved every blockage (D-059):
targeted redactions landed (tenant email out, predecessor exclusion-list
enumerations moved to gitignored `_private/` with pointers), THE_STORY.md was
approved as-is, the old form text is marked SUPERSEDED by the 2026-07-24
consent package, and rejected intake records will be kept for the pilot window
then purged in one recorded sweep. The stability bar split in two: the PUSH
bar was met and the first push executed (D-060 - origin
`atniclimate/community-connector`, plain `git push` now works); the DEPLOY bar
(Pages form + Workers relay) stays unmet until ADR-005 is accepted, the intake
pipeline works, the keygen ceremony has run, and D-023 sign-off covers the
form text. The collective checkpoint's timing is UNKNOWN, so the August
internal pilots are explicitly conditional and engineering stays on synthetic
data. Earlier the same day: the R2 EntityDetail fixes landed with a clean
adversarial round (D-057), and the D-055 sweep unit tracked the three root
docs (D-058). check-all was 11/11 green at every commit. Next action: the
ADR-005 adversarial round, then the P3.5/P3.6 intake pipeline.

## How to resume work (for the director)

1. Read HANDOFF.md - state, ordered next actions, non-negotiables. Note the
   repo is PUBLIC: every pushed commit is world-readable.
2. If the human is present: the two asks are (a) the D-023 solo correctness
   pass on docs/design/intake-consent-text-draft-2026-07-24.md (checklist
   inside, ~20 minutes, record as a DECISIONS entry), and (b) any news on
   committee timing for the bundled checkpoint + consent-text presentation.
3. If autonomous: run the ADR-005 adversarial round first (adversary wrapper,
   healthy 2026-07-24), judge and amend, then start the P3.5/P3.6 intake
   pipeline (director blueprint first - permission-adjacent at the approval
   boundary).
4. Re-arm the 8:00 AM safety cron if the usage-failover directive stands.

## Remaining open questions (defaults keep autonomous work unblocked)

**Q-CHK. Collective checkpoint timing (still unknown, D-059.9).** When known,
bundle the ask with the reviewed consent text (D-059.10). Default: synthetic
data only; August pilots conditional; no real ingestion.

**Q-TEXT. D-023 solo pass (the human's action, D-059.10).** The package awaits
your checklist review; your sign-off clears build/synthetic use. Community use
additionally waits for the committee moment. Default: nothing goes live.

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
decision session by standing rule - the public remote holds code only, never
data, so it is not the backup answer); D-032 TSDF codes primary in the UI;
D-037 in-app story authoring in v0.1; the v0 layout being client-side
deterministic; D-056.2 ownership-at-approval (unowned, facilitator-created;
owner-binding later = authority-matrix change + adversarial round); D-059.11
rejected-record retention (keep for the pilot window + one recorded purge
sweep - revisiting toward purge-on-reject is a one-line DECISIONS entry).
Revisiting any is a DECISIONS entry, not a rewrite - say the word.
