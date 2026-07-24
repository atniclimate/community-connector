# NEXT_SESSION.md - Resume Brief and Remaining Questions

> For the next session with the human present. A resuming director: read
> CLAUDE.md -> HANDOFF.md -> this file. Refresh the Brief at each session
> end (CLAUDE.md session-end protocol).

## The 60-second brief (refreshed 2026-07-24, end of the ultracode execution session)

Next-actions 1 and 2 both landed. The R2 EntityDetail defects (D-049) are
fixed and committed: effective tier now computed over the viewer-projected
attribute set in cn-perm, the detail path is I2-pure (own_settings deleted
from cn-api), the custody/tier test matrix covers all viewer classes through
to an end-to-end API test, and the provenance one-liner is structurally
single-line. The mandatory fresh adversarial round returned PASS-WITH-NOTES
with both blockers confirmed closed; its residual notes were closed in the
same commit (D-057). The long-lead gate-openers all shipped: the ADR-005
"Remote intake" DRAFT (full D-056.1 scope, round still pending), the D-023
consent-text package (banner-marked NOT FOR USE), the facilitator keygen
ceremony design, and the D-055 pre-publish sweep unit - a six-agent
world-readability scan (42 findings), machine-local content split to
gitignored `_private/`, and PLAN_1.0.md / MANIFEST.md / DEPENDENCIES.md
revised and now TRACKED, with docs/PROJECT_PLAN.md fully reconciled to
D-050..D-056. Six sweep findings need human rulings before the first push
(D-058). check-all was 11/11 green at every commit. Execution resumes at
HANDOFF next-action 1: the ADR-005 adversarial round, then the P3.5/P3.6
intake pipeline.

## How to resume work (for the director)

1. Read HANDOFF.md - state, ordered next actions, non-negotiables.
2. If the human is present: the three highest-value asks are (a) the six
   D-058 pre-push dispositions (one-line answers each), (b) scheduling the
   recorded ATNI Climate collective checkpoint (precedes the FIRST August
   ingestion, D-050), and (c) the D-023 review of
   docs/design/intake-consent-text-draft-2026-07-24.md, which now exists.
3. If autonomous: run the ADR-005 adversarial round first (adversary
   wrapper; the wrapper path was healthy 2026-07-24), judge and amend, then
   start the P3.5/P3.6 intake pipeline (director blueprint first - it is
   permission-adjacent at the approval boundary).
4. Re-arm the 8:00 AM safety cron if the usage-failover directive stands.

## Remaining open questions (defaults keep autonomous work unblocked)

**Q-PUSH. D-058 dispositions (six one-liners).** Maintainer emails in
D-003/D-011 + pii-allowlist (redact tenant address?); predecessor-PII
disclosure wording in CLAUDE.md/LAUNCH_PROMPT/migration recipe (keep,
generalize, or move specifics to _private/); THE_STORY.md approved for the
public repo?; pilot-form Parts A/C stay public as marked drafts?; accept
workspace-layout paths as-is (recommended yes); mirror carries _private/
(recommended yes). Default: no push until answered.

**Q-CHK. Collective checkpoint scheduling.** When can ATNI Climate record its
approval of the network activity? It must precede the first internal-pilot
ingestion of real people (D-030/D-050). Default: engineering proceeds on
synthetic data; no real ingestion until the recording exists.

**Q-TEXT. D-023 review loop.** The draft now exists
(docs/design/intake-consent-text-draft-2026-07-24.md, with a reviewer
checklist inside). Who reviews besides you (ATNI Climate?), and how fast can
it turn around? It also carries ADR-005's rejected-record retention question
(audit-keep vs purge - a consent-semantics call). Default: nothing goes live.

**Q-STABLE. Define "stable enough to push/deploy".** Proposed definition: R2
fixes landed (done) + fresh adversarial round clean (done) + check-all green
(standing) + ADR-005 accepted (pending) + Q-PUSH answered. Confirm or tighten
in one line. Default: that definition.

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
