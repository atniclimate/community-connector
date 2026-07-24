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
- **Least new surface**: the approval write path is the native
  `append_batch_idempotent` seam authorized by the existing
  `PermAuthorizer` (intake never calls `submit_ops`); no new authority,
  no new op kinds, no facilitator power beyond the accepted authority
  matrix (facilitator-role blueprint stands unchanged).
- **Queue formats are ADR-005 D4 verbatim**: immutable payload record +
  append-history sidecar, versioned (I7), checksummed, off-worktree.

## 1. The durable owner: native `cn intake apply`; the app is create-only
(per ADR-005 D4, round-3 amendment)

The durable op log (`OpLog`) is native-only - `cn-store`'s `log` module is
compiled out of wasm32, and the browser has no durable store. ADR-005 D4
therefore fixes the split this blueprint implements:

- **The app (browser, FSA) is CREATE-ONLY.** The facilitator grants the
  ops-directory handle once at wizard start (Chromium on the pilot PC is a
  recorded environment requirement; the environment is controlled, D-050).
  The app creates new uniquely named files only - in-app payload records,
  their initial `pending` sidecars, and decision files
  (`decisions/<record_id>.<uuid>.json`) - and NEVER rewrites an existing
  file. FSA `createWritable()` gives atomic-replace-on-close; every create
  is followed by read-back checksum verification before the UI reports
  success. A lost create is loudly visible and cheaply re-creatable by the
  supervising facilitator; a create-only writer cannot corrupt
  authoritative state.
- **Native `cn intake apply` owns every mutation.** Under the queue-root
  single-instance lock it consumes decision files through the ADR-005 D4
  admission table (binding checks, decision_id dedup by message digest,
  revision+state CAS, legal transitions). For an admitted approve,
  admission + history entry + plan + `approved_intent` are ONE atomic
  sidecar replace (never an append-then-plan sequence); the durable seam
  (shadow-state preflight) and completion follow against the real
  `OpLog`. EVERY outcome (`admitted | stale | illegal | replay`) gets a
  durable history entry BEFORE the decision file is retired into
  `decisions/consumed/` tombstones (retire-after-durable); startup
  reconciles tombstones against history and reports orphans (I12). The
  puller (mandate item 3, later) runs under the same lock. Two native
  writers serialized by the lock; app creates cannot FILE-conflict with
  either - semantic staleness is resolved by the admission table, never
  silently.
- **The app renders; it does not fold authority.** After an apply run the
  wizard prompts the facilitator to reload the group (existing load path)
  to see approved entries. The in-memory WASM `submit_ops` is NOT used for
  intake approval. The wizard surfaces staged-but-unapplied decisions and
  the apply instruction; at pilot scale this
  decide-in-app / apply-natively / reload rhythm is an acceptable and
  honest workflow.

## 2. cn-ingest becomes real (queue record model, pure logic)

`core/crates/cn-ingest` (today an empty stub) gains the filesystem-free
domain logic; ALL I/O stays in callers (app via FSA, later the puller). No
network code enters any core crate (ADR-005 D1 fence).

- `record.rs`: `QueueRecord` (immutable payload record) and `ReviewSidecar`
  exactly per ADR-005 D4 field lists, with `queue_record_version` semver
  discipline: unknown-MAJOR loud rejection, unknown-MINOR
  ignore-and-preserve ROUND-TRIPPED through serde (`#[serde(flatten)]`
  extras map, the ADR-002 pattern) - preserved across sidecar rewrites.
  `review_state` is the five-variant versioned enum
  (`pending | approved_intent | approved | rejected | failed` - `failed`
  is terminal-until-explicit-disposition, ADR-005 D4). The sidecar
  REQUIRED fields carry `record_id` and the payload record's checksum
  value (the ADR-005 sidecar-payload binding); `verify_pair()` checks both
  before any read path trusts the pair. `record_checksum`: SHA-256 over
  canonical serialization (sorted keys) of all fields except the checksum
  itself; `verify()` recomputes and compares.
- `recovery.rs`: the ADR-005 D4 legal-on-disk-states table as pure logic:
  `classify(entries) -> Vec<RecoveryAction>` (discard temp, reconstruct
  pending-only sidecar, quarantine checksum failures to `corrupt/`,
  loud-halt on binding mismatch or on a review-begun marker without a
  readable sidecar, run approval recovery on `approved_intent`, plus the
  two benign pending+marker rows: marker present with unconsumed
  decisions -> normal admission; marker present with no decision file ->
  stays pending with an I12 anomaly note). The caller is native
  `cn intake apply` (the app never mutates, so it never recovers); the
  classification and its invariants (never promote a temp, never
  reconstruct a decision history, marker-before-first-decision,
  marker create-if-absent idempotent) live here, tested exhaustively.
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
- `decision.rs`: the decision-inbox message format per ADR-005 D4
  (body-carried `decision_id` UUIDv7, record_id, payload digest,
  `expected_review_state` AND `expected_sidecar_revision`, decision type
  incl. `clear_failed`, reviewer, timestamp; versioned per I7) and the
  ADMISSION TABLE as pure logic: deterministic ordering (timestamp,
  decision_id), binding verification, dedup by decision_id + message
  digest (same id + different digest = loud conflict), CAS on
  revision+state, legal-transition enforcement, and each decision type's
  exact state/revision effect (`set_aside_note` keeps state `pending`
  but increments revision). History entries follow THE authoritative
  ADR-005 schema (decision_id, message digest, type, prior/resulting
  state+revision, reviewer, time, reason, outcome). Tests: replay ->
  recorded no-op; same-id-different-digest -> conflict; stale (state OR
  revision mismatch, incl. the pending->failed->clear_failed->pending
  ABA cycle) -> durable `stale` entry, unapplied; two concurrent
  decisions -> first admits and bumps revision, second stale - including
  when the first is note-only; replayed approve after intent -> dedup,
  never a second plan.
- `approval.rs`: the approval-transaction state machine as pure logic,
  invoked NATIVELY by `cn intake apply`:
  `plan_approval(record, template, group_id, facilitator) ->
  ApprovalPlan { preassigned_op_ids, per_op_digests, ops, batch_digest }`
  (UUIDv7 ids and canonical per-op digests generated HERE, once, at apply
  time; ops are ordinary `Operation` values - EntityCreate unowned +
  AttributeSets + EdgeCreates per the template mapping, actor =
  `cn-intake/<version>`, responsible_human = facilitator, D-056.2/ADR-005
  D5/D7). The plan's `batch_digest` uses the ADR-005 pre-link projection
  (canonical planned ops with every `intake.batch_digest` omitted), then
  populates the blocks; per-op durable-log digests are computed after
  population. Recovery follows the ADR-005 intent-as-authorization-marker
  rules: all-present completes WITHOUT re-authorization; a mixed pattern
  completes ONLY as exact contiguous prefix + absent suffix; any hole or
  out-of-order presence -> terminal `failed`; digest conflict -> terminal
  `failed`.
- **cn-store additive seam (ADR-005 D4 - lands FIRST, native-only like
  the log module):** `append_batch_idempotent(log, state, batch) ->
  per-op {absent_appended | present_same_digest |
  present_conflicting_digest}` classifying each preassigned op id against
  the DURABLE LOG (not the fold seen-set); authorization AND
  fold-acceptance are preflighted on a SHADOW CLONE of the group state in
  batch order (so AttributeSet-after-EntityCreate authorizes; any denial
  or would-be quarantine fails the batch pre-append, typed, nothing
  appended); then append-absent-only, one fsync, fold - all inside the
  apply critical section, so the real fold cannot diverge from the
  preflight. Additive API; ADR-002's formats and fold semantics
  untouched; log major unchanged. Tests: crash-simulated double-submit
  produces zero duplicate log lines; conflicting digest halts with
  nothing appended; a contiguous-prefix presence appends exactly the
  absent suffix without re-authorization; a HOLE or out-of-order
  presence goes terminal `failed` with nothing appended (negative case
  required); a batch whose preflight quarantines fails entirely with
  nothing appended.
- **cn-model additive extension (ADR-005 D5, round-4 scoping):** the
  optional `intake` block (own `intake_block_version`) on
  `ProvenanceEnvelope` (record_id, receipt_id, submission_id,
  form_version, consent digest/affirmed/affirmed_at, payload_digest,
  batch_digest). Optional serde-defaulted field + global model PATCH bump
  (accepts_schema admits same-minor; a minor bump would wrongly reject
  0.1.x data). `plan_approval` constructs the block inside the modeled
  values (Entity/AttributeInstance/Edge) it builds - the envelopes ride
  the op payloads and fold clones them, exactly as today; nothing is
  stamped at fold time. Tests: old fixtures parse under the new reader;
  every intake-created modeled value carries the block.
- **`cn intake apply` (CLI):** the durable owner. Takes the queue lock,
  runs recovery (crash-state table incl. the review-begun marker),
  consumes decision files, executes the transaction, reports (I12).
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
batch_digest, computed over the pre-link projection); recovery matrix
(none / contiguous-prefix / all present, PLUS hole and out-of-order
negative cases -> failed); hostile
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
- NO approval-write export exists in WASM (round-3 correction: the
  in-memory WASM boundary has no durable store, so a browser approval
  path would be unsound). The WASM surface is read-only compute for the
  review UI - validation, dedup classification, near-duplicate
  candidates. Plan generation, the seam, and sidecar mutation live only
  in native `cn intake apply` (section 2). The existing `submit_ops`
  stays untouched for non-intake use; intake never calls it.

Facade tests: validation/dedup/near-dup exports return correct typed
results for each viewer class; near-dup candidates projection-bounded
(no-leak, section 6). The native-side tests (seam, transaction, recovery)
live with cn-store/cn-ingest and the `cn intake apply` integration tests -
including the full decide -> apply -> reload round trip on synthetic data.

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
  conflict disposition for `Conflict` verdicts; approve / reject (reason
  required; record persists per D-059.11) / set-aside note. Every
  decision is a CREATE-ONLY decision file (section 1); the wizard then
  shows it as staged-awaiting-apply. The dashboard surfaces staged
  decisions with the `cn intake apply` instruction and, after an apply,
  prompts a group reload. Records whose sidecar shows `failed` render as
  investigation items (terminal until the facilitator's explicit
  disposition, ADR-005 D4). Recovery is native-only (`cn intake apply`);
  the wizard merely refuses to stage new decisions for a record whose
  sidecar is `approved_intent` or unreadable.
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
boundary assertion (native, at the seam/CLI integration boundary - the
facade has no approval export): ops planned by `plan_approval` and pushed
through `append_batch_idempotent` under a facilitator viewer produce
exactly the authority-matrix outcomes the facilitator-role blueprint
fixed (unowned creates allowed; nothing owner-bypassing) - asserted in
the `cn intake apply` integration tests, not assumed.

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

1. cn-store `append_batch_idempotent` seam (native, shadow-state
   preflight) + crash-simulation tests.
2. cn-model `intake` provenance block (additive, versioned) + tests.
3. cn-ingest record/sidecar/decision model + binding + review-begun
   marker + versioning + checksums + recovery classification + tests.
4. cn-ingest source/dedup/near-dup/approval logic + tests.
5. `cn intake apply` CLI (lock, recovery, decision consumption,
   transaction) + integration tests.
6. cn-api/cn-wasm read-only intake facade + no-leak extension tests.
7. app: template->form renderer (P3.6) + tests.
8. app: FSA create-only queue adapter + state module + tests.
9. app: wizard panel + review flow + decision files + smoke (full
   decide -> apply -> reload round trip on synthetic data).
10. pii-scan tripwires + self-test member.
11. fixtures: synthetic intake demo records for both demo groups; docs
    true-up (HANDOFF, MANIFEST).

Then the mandatory adversarial round on the implementation diff, judgment,
and acceptance.
