# Blueprint: Intake Pipeline - P3.5 Facilitator Wizard + P3.6 Entry Forms + Pending-Review Queue

Status: director blueprint, 2026-07-24. Implements the in-app half of the
D-053 intake architecture per ADR-005 (amended post-round-1; D-061) and the
NEXT SESSION MANDATE item 2. Permission-adjacent at the approval boundary:
implementation gets a MANDATORY adversarial round before its commits are
accepted. The remote relay + native puller (ADR-005 D2/D6, mandate item 3)
are OUT of this blueprint's scope; this blueprint builds the queue store
formats, the entry forms, and the review/approval flow they will feed.

## Design intent

The intake pipeline is the only door into the graph for pilot data (D-030):
a submission - typed in-app by the facilitator or (later) pulled from the
relay - lands as a pending-review queue record and produces graph operations
ONLY when the facilitator approves it through the normal authorized submit
path. Controlling principles:

- **The core owns every trust decision** (I2): record validation, dedup,
  near-duplicate candidates, op construction, and write authorization all
  happen in Rust; the app renders and orchestrates.
- **Least new surface**: the approval write path IS the existing
  `submit_ops` boundary with the existing `PermAuthorizer`; no new
  authority, no new op kinds, no facilitator power beyond the accepted
  authority matrix (facilitator-role blueprint stands unchanged).
- **Queue formats are ADR-005 D4 verbatim**: immutable payload record +
  append-history sidecar, versioned (I7), checksummed, off-worktree.

## 1. The browser/disk decision (the one new architectural call)

The queue root lives OUTSIDE any git worktree (ADR-005 D4), but entry and
review live in the browser app. Decision: the app reaches the queue through
the **File System Access API** - the facilitator grants the ops-directory
handle once at wizard start (Chromium on the pilot PC is a recorded
environment requirement; the pilot environment is controlled, D-050).

Durability contract, stated honestly: FSA `createWritable()` writes to a
swap file and atomically replaces the target on `close()` - the atomic-
replace half of ADR-005 D4's write primitive - but exposes no explicit
fsync. This WEAKER contract is acceptable for the in-app path and for
sidecar updates because (a) every record and sidecar carries the D4
checksum, so torn state is detected, never trusted; (b) an in-app write
failure or post-write verification failure is surfaced loudly to the
facilitator WHILE THE SOURCE IS STILL PRESENT (the person or paper form in
front of them - re-entry is cheap), unlike a remote submission whose relay
copy is deleted; and (c) the strict native protocol (temp + FlushFileBuffers
+ atomic rename) remains normative for the puller, which is the only
component that deletes upstream copies. Every in-app write is followed by a
read-back checksum verification before the UI reports success. Proposed
one-paragraph clarification to fold into ADR-005 D4 at acceptance: "the
native puller implements the strict primitive; supervised in-app writers may
substitute FSA atomic-replace-on-close plus mandatory read-back
verification, because no upstream copy is ever deleted on the in-app path."

**Concurrency without locks - writer-disjoint by construction.** The app
cannot atomically create a lock file, so the design removes shared-file
writes instead: the puller (future) only CREATES remote payload records and
their initial `pending` sidecars; the app only CREATES `in_app` records and
UPDATES sidecars. Record ids are UUIDv7 (collision-free), so no two writers
ever target the same file, except sidecars of remote records - which the
puller never touches again after initial creation. The single-instance rule
in ADR-005 D4 therefore binds the puller only; the app needs no queue lock.
The review UI re-reads a sidecar immediately before writing it and aborts on
unexpected state (optimistic check; single facilitator makes real races
operationally absent, and the append-only decision history makes even a
lost-update recoverable by inspection).

## 2. cn-ingest becomes real (queue record model, pure logic)

`core/crates/cn-ingest` (today an empty stub) gains the filesystem-free
domain logic; ALL I/O stays in callers (app via FSA, later the puller). No
network code enters any core crate (ADR-005 D1 fence).

- `record.rs`: `QueueRecord` (immutable payload record) and `ReviewSidecar`
  exactly per ADR-005 D4 field lists, with `queue_record_version` semver
  discipline: unknown-MAJOR loud rejection, unknown-MINOR
  ignore-and-preserve ROUND-TRIPPED through serde (`#[serde(flatten)]`
  extras map, the ADR-002 pattern) - preserved across sidecar rewrites.
  `review_state` is the four-variant versioned enum
  (`pending | approved_intent | approved | rejected`). The sidecar
  REQUIRED fields carry `record_id` and the payload record's checksum
  value (the ADR-005 sidecar-payload binding); `verify_pair()` checks both
  before any read path trusts the pair. `record_checksum`: SHA-256 over
  canonical serialization (sorted keys) of all fields except the checksum
  itself; `verify()` recomputes and compares.
- `recovery.rs`: the ADR-005 D4 legal-on-disk-states table as pure logic:
  `classify(entries) -> Vec<RecoveryAction>` (discard temp, reconstruct
  pending-only sidecar, quarantine checksum failures to `corrupt/`,
  loud-halt on binding mismatch or lost decision history, run approval
  recovery on `approved_intent`). Callers (the app now, the puller later)
  execute actions through their own I/O; the classification and its
  invariants (never promote a temp, never reconstruct a decision history)
  live here, tested exhaustively.
- `source.rs`: `SubmissionSource::Remote { receipt_id, ciphertext_hash,
  pulled_at, key_used, envelope_meta, relay_received_at }` vs `InApp {}` -
  the ADR's source discriminant as a sum type so remote-only fields cannot
  exist on in-app records by construction (I3: invalid states
  unrepresentable).
- `dedup.rs`: transport key `(receipt_id, ciphertext_hash)` and semantic
  key `(submission_id, payload_hash)`; `classify(new, existing) ->
  DedupVerdict { Fresh | TransportReplay | SemanticReplay | Conflict }` -
  `Conflict` (same submission_id, different payload_hash) is a distinct
  typed outcome the UI must surface for facilitator disposition, never a
  drop (ADR-005 D4).
- `near_dup.rs`: candidate scoring - normalized-name (the D-057 one-line
  normalization rules reused) plus affiliation overlap - over TWO inputs
  the caller supplies: other queue payloads and a `Projection` (already
  viewer-filtered). Pure function; conservative thresholds; every
  candidate carries its match reason (D-056.4: transparent, no
  auto-merge). Permission note (the no-leak extension, section 6): the
  graph side of the comparison is the FACILITATOR'S projection, so a
  candidate can never reference an entity the facilitator cannot already
  see.
- `approval.rs`: the approval-transaction state machine as pure logic:
  `plan_approval(record, template, group_id, facilitator) ->
  ApprovalPlan { preassigned_op_ids, per_op_digests, ops, batch_digest }`
  (UUIDv7 ids and canonical per-op digests generated HERE, once; ops are
  ordinary `Operation` values - EntityCreate unowned + AttributeSets +
  EdgeCreates per the template mapping, actor = `cn-intake/<version>`,
  responsible_human = facilitator, D-056.2/ADR-005 D5/D7). Recovery
  re-runs the same plan through the cn-store seam below; no separate
  recovery matrix is needed because the seam is idempotent by
  construction.
- **cn-store additive seam (ADR-005 D4, round-2 mandate - lands FIRST):**
  `append_batch_idempotent(batch) -> per-op
  {absent_appended | present_same_digest | present_conflicting_digest}`
  classifying each preassigned op id against the DURABLE LOG (not the
  fold seen-set), authorizing the whole batch before any append
  (all-or-nothing at the authz stage; typed batch failure on any denial
  or digest conflict, nothing appended), appending only absent ops with
  one fsync, then folding. This is the only cn-store change; ADR-002's
  formats and fold semantics are untouched (additive API, log major
  unchanged). Tests: crash-simulated double-submit produces zero
  duplicate log lines; conflicting-digest halts with nothing appended;
  partial-presence appends exactly the absent ops.
- Validation: payload field allowlist against the group template
  (types, required, enum values, length caps, Unicode normalization) via
  existing cn-model attribute validation; failures produce a typed
  validation report (I12) attached to the record, which stages as
  reviewable-but-flagged, never renders raw (ADR-005 D4 hostile-content
  rule).

Tests (all pure, no I/O): version round-trip incl. unknown-minor
preservation across sidecar rewrite; checksum tamper detection; every
`DedupVerdict` arm; conflict-not-drop; near-dup reasons present and
projection-bounded; approval plan determinism (same record + ids -> same
batch_digest); recovery matrix (none/partial/all op ids present); hostile
payloads (oversized fields, control chars, HTML, U+2028/29) neutralized.

## 3. cn-api / cn-wasm surface additions

String-in/string-out like everything else on the boundary (ADR-003);
`BOUNDARY_VERSION` minor bump. New facade methods (wasm exports):

- `intake_validate_record(record_json) -> ValidationReport` - version,
  checksum, schema, template-fit.
- `intake_dedup_check(record_json, existing_keys_json) -> DedupVerdict`.
- `intake_near_duplicates(payload_json, queue_payloads_json, viewer) ->
  candidates` - projects the graph for the viewer INSIDE the core, then
  scores; the app never receives unfiltered graph data (I2).
- `intake_plan_approval(record_json, viewer) -> ApprovalPlan` - refuses
  (typed, I3) unless the viewer resolves to an active facilitator-or-
  governance role; this is a UX-affordance check only - the REAL
  enforcement remains `PermAuthorizer` at the append seam.
- `intake_submit_approval(plan_json, viewer) -> BatchReport` - the facade
  over `append_batch_idempotent` (the ONE approval write path; the
  general `submit_ops` stays as-is for non-intake mutations). Same
  authorizer, no new authority (I2, I4).

Facade tests: plan->submit->report happy path; plan under a non-facilitator
viewer refused; resubmit-same-ids idempotency against the store (asserts
the ADR-002 dedup contract carries the recovery rule); redaction of
outcomes unchanged for the facilitator viewer class (no-leak, section 6).

## 4. P3.6 - template-driven entry forms (app, greenfield)

New `app/src/ui/forms/` renders an entry form FROM the group template
(R2 - the schema's `kinds[].attributes[]` finally gets consumed):

- `renderer.ts`: kind picker -> typed field widgets by attribute `type`
  (text, number, enum from `values`, tags, date; geo as lat/lon pair;
  link; media ref as opaque string for v0.1), `required` markers, length
  caps, `default_visibility` shown read-only (pilot entries are T1,
  D-034; per-field tier UX is post-pilot).
- Client-side validation is advisory UX only; the core re-validates at
  staging and at approval (authoritative, I2).
- Output: an inner-payload-shaped object (`submission_version`,
  app-generated `submission_id` UUID, `form_version`, consent block,
  `captured_at`, fields).
- The consent statement panel renders PLACEHOLDER-marked text until the
  D-023 sign-off lands (engineering stays on synthetic data; wiring real
  text is gated on the sign-off, CLAUDE.md real-data gate). The checkbox
  gate (unchecked = nothing stages) is implemented now because it is
  structural (D-030), regardless of final wording.

Accessibility from the start (R9): every widget labeled, keyboard
navigable, error text linked via `aria-describedby`, 375px layout.

## 5. P3.5 - facilitator wizard (app)

New `app/src/ui/intake/` mounted from `main.ts` like every panel
(`mountIntakeWizard(container, deps)`), visible only when the active
viewer resolves to facilitator-or-governance (an affordance; the core
enforces regardless):

- **Ops-directory step**: request/persist the FSA directory handle
  (IndexedDB handle storage, re-permission on session start); REFUSE a
  directory that is inside a git worktree or a known cloud-sync path
  (mirrors the puller's guard; checked by probing for `.git` ancestors
  and known sync markers, best-effort in-browser, loud on detection).
- **Queue dashboard**: pending / approved / rejected / flagged counts,
  oldest-pending age (the I12 visibility surface).
- **New entry flow**: P3.6 form -> stage as `in_app` record (payload
  record + `pending` sidecar via the section-1 write contract) -> straight
  into review view. Entry and approval stay TWO distinct recorded acts
  even when one person does both back-to-back (D-030 review-gate
  semantics; the decision history shows both).
- **Review view**: payload fields with trust-status labels
  (`source_asserted` vs local, ADR-005 D5), validation report, dedup
  verdict, near-dup candidates with reasons and side-by-side compare,
  conflict disposition for `Conflict` verdicts; approve (with the
  section-3 transaction: sidecar `approved-intent` -> `submit_ops` ->
  sidecar `approved`) / reject (reason required; record persists per
  D-059.11) / set-aside note. Recovery: on wizard start, any
  `approved-intent` sidecar triggers the `verify_recovery` flow before
  anything else is offered.
- All state flows through `app/src/state` (I4): new actions
  (`intakeQueueLoaded`, `intakeRecordStaged`, `intakeReviewDecided`, ...),
  new effects module `app/src/state/intake.ts` owning FSA I/O + wasm
  calls; reducer stays pure; no component-local mutation.

## 6. Permission boundary - the no-leak extension (why this is
permission-adjacent)

The approval boundary introduces one genuinely new leak surface:
**near-duplicate candidates computed against the graph**. The rule, and its
tests: candidates derive ONLY from the facilitator's own permission-
filtered projection (computed in-core, section 3), so review UI content
never exceeds facilitator read reach. Extend the five-class no-leak
property (cn-perm/tests/blueprint.rs pattern, cn-api-level test): for a
graph containing entities visible to governance but not facilitator, a
near-dup query for a payload matching the hidden entity returns NO
candidate for the facilitator viewer and DOES for governance. Second
boundary assertion: `intake_plan_approval` ops submitted by a
facilitator viewer produce exactly the authority-matrix outcomes the
facilitator-role blueprint fixed (unowned creates allowed; nothing
owner-bypassing) - asserted, not assumed, at the facade level.

## 7. pii-scan tripwires (ADR-005 D4, defense in depth)

`scripts/pii-scan.ps1` gains content-marker rules alongside EMAIL/PHONE/
RED PATH: `queue_record_version` (queue records), `secret-encrypted` (key
envelopes, per the ceremony design), and queue-shaped path signatures.
Positive-fixture self-test: a new check-all member (or pii-scan `-SelfTest`
flag) that GENERATES marker-bearing fixture files in the session temp dir
at runtime, asserts the scanner flags each, and cleans up - nothing
marker-shaped is ever committed, so the tripwire is proven live without
tripping itself. Documented in the script header as tripwires under the I1
process boundary, not enforcement (ADR-005 D4 honest-description rule).

## 8. Out of scope (mandate item 3 and later)

The Pages form, Worker relay, sealed-box crypto, and native puller
(ADR-005 D2/D3/D6/D8) - deploy-barred by D-059.8. This blueprint's queue
formats and approval transaction are the contract that work plugs into:
the puller writes the same `QueueRecord` with `SubmissionSource::Remote`,
and review/approval needs zero changes to serve it. Story-attribute
authoring, per-field tiers, owner-binding (D-056.2 deferral) - unchanged.

## 9. Sequencing (small verified commits, check-all green at each)

1. cn-store `append_batch_idempotent` seam + crash-simulation tests.
2. cn-ingest record/sidecar model + binding + versioning + checksums +
   recovery classification + tests.
3. cn-ingest source/dedup/near-dup/approval logic + tests.
4. cn-api/cn-wasm intake facade + boundary tests + no-leak extension.
5. app: template->form renderer (P3.6) + tests.
6. app: FSA queue adapter + state module + reducer/action tests.
7. app: wizard panel + review/approval flow + recovery flow + smoke.
8. pii-scan tripwires + self-test member.
9. fixtures: synthetic intake demo records for both demo groups; docs
   true-up (HANDOFF, MANIFEST).

Then the mandatory adversarial round on the implementation diff, judgment,
and acceptance.
