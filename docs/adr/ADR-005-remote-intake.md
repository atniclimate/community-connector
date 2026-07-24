# ADR-005: Remote Intake - Sealed-Envelope Relay and Facilitator Pending-Review Queue

- Status: DRAFT - rounds 1-3 FAIL judged and amended 2026-07-24; pending round 4
- Date: 2026-07-24 (amended three times same day after adversarial rounds 1-3)
- Phase: 3 (intake pipeline P3.5/P3.6 plus the D-053 relay; deploy gated on the
  D-059.8 deploy bar)
- Drivers: R3 (data entry and ingestion), R5 as amended by D-053 (one remote
  intake path), R10/I6 (provenance envelope on everything), I1 (no PII in the
  repo or any prompt), I7 (versioned persisted formats, unknown-major
  rejection), I12 (validation reports); rulings D-030 (the intake form is the
  individual consent instrument; form respondents only), D-034 (all pilot data
  enters at T1), D-050 (August internal pilots, convention 2026-09-14), D-053
  (intake architecture and conditionally opened gates), D-056.1 (required
  scope of this ADR), D-056.2 (ownership-at-approval), D-056.4 (dedup and
  near-duplicate surfacing, risk register), D-059.11 (rejected-record
  retention)
- Adversarial round 1: FAIL (gpt-5.6-sol, 2026-07-24, review lane) - five
  blockers: browser-bundle trust root, Windows crash/approval transaction
  protocol, relay-to-queue idempotency, consent/audit contract completeness,
  key-custody claims. All five judged valid and addressed by amendment.
- Adversarial round 2: FAIL (same lane, on the amended text at cc20638) -
  two new blockers (the approval recovery assumed durable op-id semantics
  cn-store does not provide; the KV POST-counter/hard-cap reconciliation is
  not implementable on eventually consistent KV) plus four majors
  (crash-state table, bundle measurement procedure, sidecar binding and
  approved-record closeout, rotation cutoff proof). All judged valid and
  addressed by the second amendment: the durable idempotent batch-append
  seam (D4), the receipt ledger replacing the counter (D6), the crash-state
  table (D4), the canonical manifest procedure (D8), sidecar-payload
  binding and the decided approved-record closeout (D4/D5), and the
  relay-side fingerprint admission cutoff (D3/D6). The keygen ceremony
  companion was amended in the same commit to match (bundle check,
  destruction timing, custody phrasing).
- Adversarial round 3: FAIL (same lane, at 3cb0ede) - three blockers: the
  durable seam was unreachable from the browser/WASM approval path (OpLog
  is native-only), recovery re-authorization could mislabel durable ops as
  pending (and fold quarantine could partially approve), and the two-write
  KV ledger had no atomic admission, no observation window, and no cutoff
  epoch. Four majors: orphan-vs-lost-history indistinguishable plus a
  degraded path that deleted the only durable copy; residual hard-cap
  claims; non-executable manifest ceremony plus service-worker gap;
  post-sweep evidence missing the consent affirmation and unrepresentable
  in the current provenance schema. All judged valid. Third amendment: the
  durable approval owner is NATIVE `cn intake apply` (D4 - the app becomes
  create-only staging), intent-as-authorization-marker recovery with a
  terminal `failed` state, ledger write-order/precedence/TTL-interval
  rules and a cutoff epoch, the review-begun marker, retain-on-degraded-
  durability, canonical manifest grammar + ceremony steps + no-service-
  worker rule, and a versioned structured intake-provenance block carrying
  the consent affirmation with a survival contract for the sweep manifest.

## Context

v0.1.0 must accept real submissions from convention attendees and August
internal-pilot participants who scan a QR code on their own phones (D-030,
D-050). The hard constraints, all previously decided:

- The intake form is the individual consent instrument; only form respondents
  enter the graph, and every submission passes a facilitator review gate
  before ingestion (D-030, D-053).
- No real person's PII may enter this repository, any commit, any fixture, or
  any Codex prompt - the prime directive outranks every feature goal (I1).
- No external form platform (no Google, no Microsoft): a third party would
  hold readable personal data (D-053).
- No authentication for submitters in v0.1.0 or v1.0 (D-053) - the submit
  endpoint is public. (The facilitator's pull/delete control plane IS
  authenticated; "no auth" is a ruling about respondents, not operators.)
- The graph is local-first and never listens on the network; R5's stance is
  "network-ready, not networked", and choosing protocols or identity
  standards is human-gated.

The human resolved the resulting tension in the 2026-07-24 gate-grill session
(D-053) by choosing a sealed-envelope relay: the only server in the path
stores ciphertext it cannot read, and the only machine that decrypts in
operation is the facilitator's pilot PC. The human conditionally opened three
gates for exactly this path: the public remote for the GitHub Pages form, the
Cloudflare Workers hosting vendor, and the associated spend - with
preconditions before any push or deploy (the push bar was met and executed,
D-060; the deploy bar D-059.8 remains).

D-053 trips the architecture-stance-change rule three ways: it amends R5, it
introduces a new persistent store (the pending-review queue), and it
introduces new persisted formats. D-056.1 therefore requires this ADR plus
one adversarial round, and fixes its scope. This document is that ADR,
amended after round 1.

## Decision

### D1. Intake is INGEST, not sync - the R5 amendment is scoped to one inbound path

Remote intake is classified as ingestion (R3), not synchronization (R5):

- The puller is intake tooling. It routes through cn-ingest concepts
  (validation, provenance stamping, dedup, staged review) and submits
  approved data through the same authorized write path as any other ingest.
  It never touches the SyncTransport seam: no SyncTransport implementation,
  SyncFrame kind, or capability descriptor is added, referenced, or
  implemented by any intake component (ADR-002 D8/A-B8 stand unmodified).
- The graph and core never listen on any network. The pilot PC initiates
  outbound HTTPS pulls; no component of this system accepts an inbound
  connection on the pilot PC.
- The relay stores ciphertext only. It cannot read, decrypt, validate,
  index, or transform payloads; its API is store-blob and authenticated
  fetch/delete (contract in D6).
- The R5 amendment is exactly this narrow: one pull-based ciphertext relay
  for intake. Peer sync, federation, identity standards, and every other
  networking decision remain human-gated and untouched by this ADR.

**Normative module fence (where networking is allowed to exist).** All
network I/O for intake lives in ONE intake adapter at the CLI/tooling edge
(the puller component). The adapter fetches ciphertext and hands bytes to a
network-free intake facade (decrypt, validate, stage). `cn-model`,
`cn-schema`, `cn-store`, `cn-perm`, `cn-graph`, `cn-sync`, `cn-api`, and
`cn-wasm` remain network-free: no HTTP client, socket, or network dependency
may be added to any of them for intake. Whether the adapter ships as a `cn`
CLI subcommand (leaning yes, for provenance and cn-ingest reuse) or a
separate small binary is an implementation choice; the fence and the actor
identity (D5) are not. Any diff adding network I/O outside the intake
adapter fails review against this ADR.

### D2. Architecture flow

```
attendee phone            GitHub Pages          Cloudflare Workers + KV      pilot PC
+-------------+          +--------------+          +------------------+          +---------------------------+
| scan QR     |  serves  | static intake|  HTTPS   | ciphertext relay |  HTTPS   | puller:                   |
| fill form   | <------- | form (UI     |  POST    | (blob store the  |  pull    |  fetch -> decrypt locally |
| seal box in |          | only; no     | -------> | server cannot    | <------- |  -> durable stage into    |
| the browser |          | secrets, no  |          | read; TTL, size  |  then    |     pending-review queue  |
+-------------+          | data at rest)|          | cap, rate limit) |  delete  |  -> wipe relay receipt    |
                         +--------------+          +------------------+          +---------------------------+
```

- The QR code resolves to a static intake form hosted on GitHub Pages from
  the public repo. Pages is interface only: it holds no secrets and runs no
  backend. Submission content is encrypted in the browser before it leaves
  the phone, so no readable submission data rests on Pages infrastructure -
  subject to the browser trust model in D8, which is what makes this claim
  meaningful rather than assumed.
- The browser seals the submission to the facilitator public key (D3) and
  POSTs only ciphertext to the relay.
- The relay is a minimal Cloudflare Worker writing to KV: store the blob
  under a server-generated receipt id, return the receipt id, expire
  unpulled blobs by TTL (D6).
- The pilot PC puller fetches ciphertext, decrypts locally, stages the
  submission durably into the pending-review queue (D4), and only after
  verified durable staging deletes that receipt from the relay.
- Fallback when the relay or form is unavailable: direct in-app facilitator
  entry, which is the primary intake path anyway (D-053).

### D3. Sealed-envelope payload format, versioning, and key custody (I7)

**Cryptography.** Submissions are libsodium sealed boxes
(`crypto_box_seal`) to the facilitator's X25519 public key: the sender uses
an ephemeral key pair, so only a holder of the facilitator private key can
decrypt, and submissions are unlinkable to sender keys (the form itself is
the consent instrument, so sender anonymity at the crypto layer is
acceptable, D-053). Sealed boxes authenticate the ciphertext, not any
adjacent cleartext; see "outer fields are hints" below. Sealed boxes have no
forward secrecy against later compromise of the recipient key; the exposure
model below states this plainly.

**Concrete distribution.** The form vendors `libsodium-wrappers` (the
standard build; sealed boxes need no sumo build) at one pinned exact
version, bundled at build time into the form artifact - no CDN, no runtime
fetch of any code. The vendored artifact's version, provenance (npm
registry, package integrity hash), and license are recorded in the deploy
manifest (D8). Bundle integrity is established by the D8 full-bundle pin,
not by subresource integrity (SRI cannot defend the page that carries the
integrity attribute).

**Key custody - stated exactly.** The ACTIVE private key is used only on
the pilot PC. Two controlled offline recovery copies also exist, created at
the keygen ceremony (companion design, D-056.4): a printed plaintext sheet
in a sealed envelope in a locked location, and a passphrase-encrypted USB in
a second location. Those copies are decrypt-capable if restored; custody of
them is part of the ceremony's storage rules. The private key in any form
never appears in the repo, the relay, the Pages bundle, any commit, or any
Codex prompt. The public key and its fingerprint are the only key material
that is world-readable.

**Exposure model (adopts the companion ceremony's model).** Three distinct
events, three distinct blast radii:

- **Key LOSS** (pilot PC dies, backups unrecoverable): in-flight and future
  submissions sealed to that key are unreadable; nothing is disclosed.
  Remedy: rotate, re-solicit.
- **Key LEAKAGE** (private key disclosed, PC otherwise intact): the leaked
  key decrypts any ciphertext the attacker holds or later obtains -
  everything still on the relay, anything they captured historically in
  transit or from the relay, and new submissions sealed to the old key
  until the form is stopped or rotated. Sealed boxes provide no forward
  secrecy against this. Remedy: emergency rotation (below) plus notify per
  the pilot's incident expectations.
- **Full pilot-PC COMPROMISE**: exposes the unlocked active key, the
  decrypted queue (all pending/approved/rejected records in the window,
  per D-059.11), the pull/delete credential, and local graph data. This is
  strictly worse than key leakage and is NOT bounded to "already pulled"
  submissions. Mitigations are the ceremony's platform requirements (disk
  encryption, dedicated account, no remote access) and D4's at-rest rules;
  the residual risk is accepted for v0.1.0 and recorded here.

**Envelope format.** Two layers, both explicitly versioned (I7):

- Outer envelope (cleartext to the relay, deliberately minimal):
  `intake_envelope_version` (semver), `recipient_key_fingerprint`, and the
  sealed-box ciphertext (base64). Nothing else; the relay learns format
  version, claimed target key, size, and timing - no content.
- Inner payload (plaintext only after local decryption on the pilot PC):
  `submission_version` (semver), `submission_id` (client-generated UUID,
  the SEMANTIC dedup key - D4), `form_version`, `consent` block
  (`consent_text_digest` of the exact consent statement displayed,
  `consent_affirmed` which MUST be true for the form to send,
  `consent_affirmed_at`), `captured_at` (client capture timestamp), and the
  form fields. Every inner field is a CLIENT ASSERTION: the endpoint is
  anonymous, so a payload can be constructed without the form. The queue
  and provenance record these values as `source_asserted`, never as
  verified fact (D5); their evidentiary value is that the facilitator
  reviews them against expectations before approval.

The puller rejects unknown MAJOR versions of either layer loudly and records
them in the validation report; unknown MINOR fields are ignore-and-preserve
(the ADR-002 version policy, applied to intake formats).

**Outer fields are untrusted routing hints.** The puller uses
`recipient_key_fingerprint` only to CHOOSE which held key to try first; it
verifies by successful authenticated decryption, and the queue record stores
the fingerprint of the key that actually opened the box (`key_used`). A
fingerprint naming a key the puller does not hold is a loud typed error
(I3); the receipt and ciphertext are RETAINED on the relay (never deleted)
while it is investigated. Tampered outer fields can at worst cause loud
denial or misrouting, never silent loss.

**Key pinning.** The puller pins the facilitator key fingerprint AND the
deployed-bundle hash (D8) in local, off-repo config written at the ceremony.
Before any pull run it verifies: deployed form bundle hash against the
bundle pin, the form's embedded public key fingerprint against the key pin,
and the key pin against its own local secret key's public half. Any
mismatch: halt loudly, decrypt nothing, stage nothing, DELETE NOTHING on
the relay (evidence preservation), no override flag - per the companion
ceremony design.

**Rotation protocol (drain-before-flip, with an enforceable cutoff).**
Rotation is never a simple form flip, because cached pages and open tabs
can produce old-key ciphertext after a redeploy. The enforcement point is
the relay: the Worker validates the outer `recipient_key_fingerprint`
against its configured admission allowlist (D6) - a cleartext field, so no
content access is needed - which makes "no more old-key envelopes" a
provable state rather than an estimate:

1. Pause distribution: stop presenting the QR / announce the pause window.
2. Drain: pull and reconcile the relay via the D6 receipt ledger until
   every ledger entry is accounted for, or relay TTL has elapsed since
   pause.
3. Run the new keygen ceremony; deploy the new form; independently verify
   the deployed bundle against the D8 pinned manifest.
4. Cutover: add the new fingerprint to the relay admission allowlist;
   after the drain window, REMOVE the old fingerprint. From that moment
   the relay rejects old-form POSTs with a typed "form out of date -
   reload" response: an arbitrarily old open tab gets a VISIBLE failure,
   never a silently lost submission.
5. Retain old-key decrypt capability for one relay TTL past the cutover
   (envelopes admitted before cutover can still be pulled and decrypted;
   the queue records `key_used`).
6. Destroy the old key and its backups only after that TTL has elapsed
   AND ledger reconciliation shows no unaccounted old-key receipt.
   "No old-key envelope can still arrive" is then enforced by admission,
   not assumed from an open-tab allowance.

Accepted residual: a submitter on a stale tab after cutover must reload
and resubmit; their attempted submission is refused visibly, not lost
silently.

Emergency rotation on suspected key leakage or PC compromise: execute
immediately from a machine believed clean; do NOT trust the compromised
host's queue, pins, or pull/delete credential; rotate the Worker credential
(D6) in the same action; the exposure model above governs what must be
assumed disclosed.

### D4. Pending-review queue: outside the op log, outside the worktree, durable-write before relay-wipe

**Placement - two senses, both normative.**

*Outside the op log.* The queue is NOT part of the cn-store op log, and
pending submissions produce no operations. Rationale: the op log is
append-only domain history; rejected submissions must leave no trace in
graph history (D-030 consent semantics); queue records carry raw,
pre-validation data with no entity ids and no place in op schema; and
keeping unreviewed PII out of the op log keeps it structurally unreachable
by export and sync machinery (ADR-002 A-B5). Only facilitator-approved
submissions generate operations, through the normal authorized submit path.

*Outside the worktree.* The canonical queue root is the facilitator ops
directory OUTSIDE any git worktree (the same off-repo area the keygen
ceremony uses for key material). A repo-local staging directory is NOT an
option (round-1 amendment; the prior draft allowed it). The puller refuses
to run - typed error, I3 - if the resolved queue root lies inside any git
worktree or a known cloud-sync directory (Dropbox/OneDrive/Drive paths),
checked at startup.

**At-rest controls (the queue is T1 operational data, not just a
directory).** Preconditions checked by the puller at startup and recorded
in each run report (I12): full-disk encryption active on the volume
(BitLocker on the pilot PC - the ceremony's platform requirement, made a
queue precondition here); queue root ACL'd to the facilitator account;
Windows Search indexing excluded for the queue root; no backup/sync agent
covering it. There is NO additional queue backup: a second PII copy is a
worse privacy trade than re-solicitation. Consequence, accepted and
recorded: after a receipt is deleted from the relay, that submission exists
only on the pilot PC's disk; single-disk loss between pull and approval
loses it, and the remedy is re-solicitation. (Approved submissions survive
as ops in the graph; the queue copy is audit trail.)

**Persisted format (I7).** One immutable payload record per staged
submission plus one small review-state sidecar, both carrying
`queue_record_version` (semver). Readers reject unknown MAJOR versions
loudly; unknown MINOR fields are ignore-and-preserve, including across
sidecar rewrites. The payload record is written once at staging and never
modified; it holds:

- `record_id` (puller-generated UUIDv7) and `staged_at` (local clock);
- `source`: `remote | in_app` - the queue is the ruled landing zone for
  BOTH paths (CLAUDE.md real-data gate); envelope/relay fields below are
  present for `remote` and absent for `in_app`;
- outer-envelope metadata as received, marked asserted: claimed
  fingerprint, envelope version;
- `key_used`: fingerprint of the key that actually decrypted (remote only);
- `receipt_id`, `relay_received_at` if the relay reports it, `pulled_at`
  (local clock), `ciphertext_hash` (SHA-256, computed by the puller before
  decrypt) - remote only;
- the decrypted inner payload verbatim (all fields `source_asserted`,
  including the consent block) and `payload_hash` (SHA-256 over a
  canonical serialization of the inner payload);
- `record_checksum` over the record's own content (torn-write detection).

The sidecar holds mutable review state: `review_state`
(`pending | approved_intent | approved | rejected | failed` - the
write-ahead state AND the terminal investigation state are part of the
versioned persisted enum, not implementation details), the full decision
history (array of `{state, reviewer, decided_at, reason}` - never
overwritten, only appended to), reviewer notes, validation-report
reference, near-duplicate candidates surfaced and the facilitator's
disposition, and - in `approved_intent` and beyond - the approval
transaction fields below. `failed` is terminal-until-investigated: it
records a digest conflict or partial-inconsistency outcome, blocks any
retry, and only an explicit facilitator disposition (a history-recorded
transition) can return the record to `pending` - a retry can therefore
never sidestep conflict evidence by regenerating fresh op ids. Sidecar
updates use the atomic-replace protocol below, and the history array means
a rewritten sidecar still carries every prior decision.

**The durable owner and the create-only app (round-3 amendment - this
supersedes any browser-side approval write path).** The durable op log
(`OpLog`) is native-only by design; no browser or WASM component owns a
durable store. The write-authority split is therefore:

- *The app (browser, FSA) is CREATE-ONLY.* It may create new files with
  unique names - in-app payload records, their initial `pending` sidecars,
  and DECISION FILES (`decisions/<record_id>.<uuid>.json`: record id,
  payload digest, the facilitator's decision, reviewer, timestamp) - and
  never rewrites any existing file. A lost or torn created file is
  re-creatable by the supervising facilitator and is loudly visible
  (read-back verification), and a create-only writer cannot corrupt
  authoritative state.
- *Native `cn intake` owns every mutation.* The puller stages remote
  records; `cn intake apply`, run under the queue-root single-instance
  lock, consumes decision files: appends each decision to its record's
  sidecar (verifying the sidecar-payload binding and the decision file's
  payload digest first), executes approvals through the transaction below
  against the real `OpLog`, and completes sidecars. Sidecar rewrites, op
  appends, fsync, and fold all happen inside this one native critical
  section - the seam is invoked only where the durable store exists.
- *The app renders; it does not fold authority.* After an apply run the
  facilitator reloads the group in the app (the existing load path) to see
  approved entries; the wizard surfaces staged-but-unapplied decisions and
  prompts for the apply step. The in-memory WASM `submit_ops` path is NOT
  used for intake approval.

Concurrency collapses to one rule: the lock serializes the two native
writers (puller, apply); the app's create-only writes cannot conflict with
either by construction (unique names, no rewrites).

**Sidecar-payload binding.** Filename adjacency is not an integrity
binding. The sidecar REQUIRED fields include the `record_id` and the
payload record's `record_checksum` value (its digest); every reader
verifies both against the payload record before trusting the pair, so a
swapped, misnamed, or stale sidecar is a loud typed error (I3), never a
silent mis-association. The payload record carries its `record_id`; the
expected sidecar is the one bearing that id and a matching payload digest.

**Windows crash-state protocol (replaces round-0's four verbs).** All queue
writes follow one primitive: write to a temp file in the queue directory
(exclusive create, unique name, SAME VOLUME as the final path), flush to
disk (`FlushFileBuffers` via the runtime's fsync), close, then atomically
rename onto the final path (`MoveFileEx` with
`REPLACE_EXISTING | WRITE_THROUGH` semantics - "atomic" here means
replacement visibility; power-loss durability rests on the file flush plus
startup recovery), then flush the directory handle where the platform
allows. Checksums are computed over a canonical serialization (sorted
keys, defined with the format schema). A single-instance lock file at the
queue root (exclusive create; stale-lock takeover only with
process-liveness check) forbids concurrent pullers/review processes - two
instances is a refuse-to-run, not a race.

Per-receipt staging sequence: stage payload record via the primitive ->
stage `pending` sidecar via the primitive -> re-read and checksum-verify
both -> only THEN issue the relay delete for that receipt. A crash anywhere
before the delete leaves the receipt on the relay; the next run re-pulls it
and transport dedup (below) makes re-staging a recorded no-op. A delete
whose acknowledgement is lost is retried; deleting an already-deleted
receipt is idempotent at the relay (D6).

**Legal on-disk states and deterministic recovery (round-2 amendment).**
Startup recovery scans the queue root and resolves every state by rule -
no state is left to operator judgment:

The distinction between "sidecar never existed" and "sidecar existed and
was lost" needs a durable fact, because the payload file alone is
observationally identical in both cases (round-3 amendment): before the
FIRST decision beyond initial `pending` is written for a record, the
mutating writer creates a write-once REVIEW-BEGUN MARKER
(`<record_id>.reviewed`, empty content, atomic primitive). The marker is
never deleted while the record lives; its presence without a readable
sidecar proves lost decision state.

| Found | Meaning | Action |
|---|---|---|
| Temp file | Torn write | Discard. Never promote a temp file; its content, if it mattered, is re-creatable (relay re-pull or in-app re-entry). |
| Payload record, no sidecar, NO review-begun marker | Crash between the two staging writes | Reconstruct a `pending` sidecar (initial state ONLY). Remote records: the relay receipt still exists (delete is gated on sidecar verify), so re-pull would also heal this; transport dedup absorbs the overlap. |
| Payload record, no readable sidecar, review-begun marker PRESENT | Lost decision state | Loud halt for facilitator disposition - a decision history is never silently recreated as `pending`. |
| Payload or sidecar failing checksum | Torn/corrupt final | Move the pair to a `corrupt/` subdirectory (retained, never trusted, swept with the window close), loud typed error (I3), and DO NOT delete the relay receipt for that record - local durability is unproven, so the upstream copy is preserved. |
| Sidecar whose binding fields mismatch its payload | Mis-association | Loud typed error; both files retained for facilitator investigation; no automatic repair. |
| Sidecar in `approved_intent` | Crash during approval | Run the approval recovery rule (above) before any other queue work. |
| Directory-handle flush unavailable | Degraded platform | Staging proceeds for LOCAL purposes with a WARN recorded in the run report (I12), but the relay delete for affected receipts is DEFERRED - read-back proves visibility, not power-loss namespace durability, so the upstream copy is retained until a later run achieves a fully flushed staging of that receipt (re-pull + transport dedup make this cheap); if the platform never allows it, the TTL bounds the retention and the receipt is treated as never-safely-staged. "Durable-write before relay-wipe" is never weakened. |

**Approval transaction (queue -> op log) - on a durable idempotent seam
(round-2 amendment).** Round 2 established that ADR-002's op-id dedup is a
FOLD property (an in-memory seen-set that makes re-application a state
no-op), not a durable-log property: today's `append_batch` serializes
blindly, today's `submit` reports an already-seen op as `Applied`, and
"exists in the durable log", "is in the fold's seen-set", and "is
quarantined" are three different facts. Recovery built on fold semantics
would duplicate audit-log lines. This ADR therefore REQUIRES a new
additive cn-store seam, which the intake implementation lands before any
approval code:

*Idempotent durable batch append.* Input: an ops batch with preassigned
ids and a canonical per-op digest. The store classifies each op id against
the DURABLE LOG (not the fold seen-set) as
`absent | present_same_digest | present_conflicting_digest`. Any
`present_conflicting_digest` -> typed batch failure, nothing appended
(the dangerous same-id-different-bytes case, detected instead of
undefined). Authorization and fold-acceptance are PREFLIGHTED on a shadow
state in batch order (round-3 amendment): the ops are authorized and
applied sequentially against a CLONE of the current group state - so an
`AttributeSet` following its `EntityCreate` authorizes correctly, which a
whole-batch check against the pre-batch state cannot do - and any denial
OR any would-be quarantine in the preflight is a typed batch failure with
nothing appended. Only a preflight-clean batch appends (absent ops only),
fsyncs once, and folds; because append happens inside the same native
critical section as the preflight, the real fold cannot diverge from the
preflight fold - post-append quarantine and partial semantic approval are
impossible by construction, not merely reported.

The transaction (all steps inside `cn intake apply`'s critical section):

1. On consuming an approve decision, the apply step generates the plan
   (preassigned UUIDv7 op ids, canonical per-op digests, `batch_digest`)
   and updates the sidecar to `review_state: approved_intent` carrying the
   complete plan and the reviewer/timestamp history entry. This durable
   intent record IS the authorization marker: it certifies that authority
   was decided at intent time.
2. The batch goes through the seam (preflight on shadow state, authorized
   by the normal `PermAuthorizer` - no new write authority; append-absent;
   fsync; fold).
3. On success, the sidecar is updated to `approved` with the submit report
   reference. On FIRST-ATTEMPT typed failure (preflight denial or
   quarantine - nothing appended), the sidecar returns to `pending` with
   the failure in the decision history: loud, reviewable, no partial
   state. On digest conflict, the sidecar goes to `failed` (terminal
   investigation state) - never `pending`.

Recovery (a sidecar found in `approved_intent` at startup, handled before
any other queue work): classify the plan's op ids against the durable log.

- ALL `present_same_digest`: the append completed before the crash.
  Complete the sidecar to `approved` WITHOUT re-authorization - authority
  was decided and durably marked at intent time; re-deciding it against
  post-crash state could mislabel already-durable graph mutations as
  pending, which is the round-3 blocker this rule closes.
- ALL `absent`: nothing was appended. Re-run step 2 under the intent's
  existing plan (same ids and digests - never regenerated).
- MIXED absent / present_same_digest: a durable prefix exists (the log
  writes lines sequentially and recovery-truncates only a torn final
  line). The intent marker authorizes COMPLETION: append exactly the
  absent suffix, fsync, fold, complete to `approved`. No
  re-authorization - the marker, not post-crash authority, governs.
- ANY `present_conflicting_digest`: sidecar to `failed`, loud typed error,
  facilitator investigation. Nothing appended, nothing deleted.
- If the sidecar itself cannot be rewritten (I/O failure), the apply run
  HALTS loudly (I3) rather than proceeding to other work - a repeatedly
  encountered `approved_intent` is a stop condition, not a loop.

Rejection is a single sidecar update (`rejected` plus history entry);
records then persist under D-059.11 until the recorded purge sweep.

**Dedup - transport and semantic layers (round-1 amendment).**
`submission_id` is client-controlled and therefore cannot be the only key:

- *Transport dedup* (re-pull safety): key `(receipt_id, ciphertext_hash)`.
  A re-pulled receipt matching an existing record is a recorded no-op
  (I12). The same receipt id with a DIFFERENT ciphertext hash is a loud
  transport-integrity conflict (I3): both ciphertexts are retained, the
  relay copy is NOT deleted, and the facilitator is alerted - it means
  relay-side substitution or a platform fault, never a no-op.
- *Semantic dedup* (duplicate submissions): key
  `(submission_id, payload_hash)`. Identical pair -> recorded replay,
  no-op. Same `submission_id` with DIFFERENT `payload_hash` -> a loud
  CONFLICT surfaced to the facilitator for disposition (both records are
  staged and linked); it is never silently dropped (I3/I12) - it is either
  a client bug or someone racing a submission id.
- *Near-duplicate surfacing* (D-056.4, no auth means same person, new
  UUID): the review UI surfaces near-duplicate candidates
  (normalized-name plus affiliation similarity against both the queue and
  the existing graph) at review time, with the match reason shown.
  Conservative matching at pilot scale (~150-300 records); the facilitator
  decides approve / reject / merge-by-hand; tooling never auto-merges.

**Hostile decrypted content.** Decryption is followed by strict schema
validation (field allowlist, type checks, length caps, Unicode
normalization per the D-057 one-line rules) BEFORE anything renders. The
review UI renders all submission content as escaped text - never as HTML;
exports escape likewise. A payload failing validation stages as a record
with a validation-failed marker (reviewable, rejectable) rather than
rendering raw.

**Retention and lifecycle - all states, decided now (round-2 amendment).**
Rejected: per D-059.11, kept in full for the pilot window, then purged in
ONE RECORDED SWEEP at window close. Pending at window close: reviewed to a
decision or purged in the same sweep - the queue is never left holding
undecided PII after the window. Approved: PURGED in the same sweep - there
is no archive branch. Rationale: privacy first - no decrypted PII store
outlives the window; the data itself lives on as governed graph content,
and consent linkage survives via the D5 intake provenance block - which
carries the affirmative consent assertion and its asserted time
(`source_asserted`), the consent-text digest, and the linkage digests,
not merely the instrument's version. The exact consent text stays
resolvable because every deployed consent text's digest and full wording
are recorded at deploy in the ops log and referenced from the D-023
sign-off record.

**Sweep-manifest survival contract (round-3 amendment).** What survives
after the sweep is designed evidence, not a leftover: the sweep manifest
(dated, non-PII) maps each record id to its payload and batch digests,
decision summary (state/reviewer/timestamp), consent digest + affirmation
+ asserted time, op-id list, and sweep operator. Integrity and
loss-detection: the manifest's SHA-256 and record count go INTO the
repo-committed DECISIONS.md sweep entry - a public, version-controlled
anchor - so a lost or altered off-repo manifest is detectable against a
durable record. Retention: the manifest is kept as long as the graph data
it evidences exists, under the facilitator ops log's ACLs; it contains no
PII, so indefinite retention violates nothing. `corrupt/` quarantine
content is purged in the same sweep.

**PII containment - honest description (round-1 amendment).** The
protections here are layered process plus tripwires, with I1 as the human
boundary - not a mechanical guarantee:

- The canonical queue root is outside any worktree, and the puller refuses
  worktree/sync paths (the structural layer).
- pii-scan gains queue tripwires: the `queue_record_version` marker, the
  `secret-encrypted` key-envelope marker (per the ceremony design), and
  queue-shaped path signatures, each with positive-fixture tests proving
  the scan actually fires; staged mode is exercised by the pre-commit
  hook.
- These are DEFENSE IN DEPTH. A stripped-marker copy, a bypassed hook
  (`--no-verify`, uninstalled per-clone hook), or hand-copied text defeats
  them; what stands behind them is the I1 process boundary the humans and
  sessions enforce. The previous draft's claim that a misplaced queue file
  "cannot pass the pre-commit hook" is withdrawn as overclaim.

### D5. Intake provenance envelope (I6) - with trust status, in a schema
that can actually carry it

**The carrier (round-3 amendment).** The current `ProvenanceEnvelope`
(origin, actor, responsible human, recorded time, custody, schema version)
has no fields for intake linkage, so this ADR mandates an ADDITIVE,
VERSIONED extension in cn-model: an optional `intake` block on the
provenance envelope (present exactly on intake-produced entities/edges;
absent everywhere else; unknown-minor tolerated per I7; envelope schema
minor bump, no breaking change). The block carries: queue `record_id`,
relay `receipt_id` (remote only), `submission_id`, `form_version`,
`consent_text_digest`, `consent_affirmed` and `consent_affirmed_at` (both
explicitly `source_asserted` - the individual consent instrument's
affirmative assertion and its claimed time survive into graph provenance,
not merely the instrument's version), `payload_digest`, and
`batch_digest`. The intake blueprint implements this cn-model change; it
is intake scope, not a change to ADR-002's op format (the envelope is
built at fold time as today).

Every entity and edge produced by an approved submission carries the full
provenance envelope. Round-1 amendment: values are recorded WITH their
trust status - `source_asserted` (client-controlled claims), or locally
observed (`relay_observed` / `facilitator_observed`) - so I6 is meaningful
rather than merely populated:

- `actor`: the intake tooling as a software-agent identifier with version
  (e.g. `cn-intake/<semver>` - stable regardless of final packaging), not
  the facilitator.
- `responsible_human`: the reviewing facilitator - required because the
  actor is non-human (ADR-002 D2), and correct because approval is the act
  that admits the data.
- `captured_at` (source_asserted): when the person says they filled the
  form. Recorded verbatim; clock-implausible values (future, or preceding
  the form's deploy) raise a validation warning; NEVER used for ordering
  or retention decisions - `pulled_at` and the approval timestamp
  (facilitator_observed) govern those.
- `form_version` (source_asserted): the form the submitter claims they
  saw; paired with the consent block's `consent_text_digest`, which the
  puller checks against the digests of known deployed consent texts - a
  match is evidence (not proof) the shown text was one we deployed.
- `receipt_id` (relay_observed): the custody-relevant relay hop.
- `submission_id` (source_asserted): the semantic dedup key.
- The queue `record_id` plus the payload and batch digests: during the
  pilot window these resolve to the full queue audit record (decision
  history, hashes); after the recorded close-of-window sweep the full
  record is purged BY DESIGN (D4 lifecycle), and the identifiers/digests
  here plus the sweep manifest ARE the surviving audit evidence - the
  link is an identifier that outlives its referent, stated plainly rather
  than promised as forever-resolvable.

Tier: every pilot entry enters at T1; the tier authority is ATNI Climate
(D-034). Per-field tier UX is post-pilot work.

### D6. The relay: API security contract and threat model

**API contract (round-1 amendment - previously implicit).**

- `POST /submit` (public, no auth): accepts one outer envelope within the
  size cap, and only if its `recipient_key_fingerprint` is on the Worker's
  configured admission allowlist (cleartext field; this is also the D3
  rotation cutoff - an out-of-date form gets a typed "reload the form"
  rejection, never silent loss). The Worker GENERATES the receipt id
  (128-bit random, server-side; the client can neither choose nor predict
  KV keys). KV offers no strongly consistent conditional create, so
  collision is prevented by ID-SPACE RANDOMNESS, not by a primitive: at
  pilot scale the overwrite probability is negligible (order 2^-128 per
  pair) and is stated as negligible, not impossible. Response: the receipt
  id and nothing else. The acknowledgement MEANS "the relay
  stored your sealed envelope"; it does not mean staged, reviewed, or
  approved - the D-023 confirmation wording must say so (see TTL semantics
  below).
- `GET /receipts` + `GET /blob/{id}` + `DELETE /blob/{id}` (authenticated
  control plane, pilot PC only): list is paginated and repeatable; reads
  are repeatable (no destructive read - "fetch-then-delete" from the
  round-0 draft is withdrawn; fetch and delete are separate, and delete
  happens only after durable staging per D4); delete is idempotent
  (deleting a missing id succeeds with a distinct code); a delete whose
  response is lost is safely retried. KV list/read/delete is NOT assumed
  transactional; the puller's reconciliation tolerates eventual
  consistency (a just-deleted receipt may still list; re-delete is a
  no-op; transport dedup absorbs re-reads).
- No public read, status, or delete surface exists. Missing-id and
  unauthorized responses are indistinguishable to an unauthenticated
  caller (no existence oracle; receipt ids grant no lookup capability).

**Control-plane credential.** A single bearer credential authorizes
list/read/delete. The Worker stores only a hash of it (verifier model,
constant-time compare) as a Worker secret - resolving the round-0
contradiction ("held only by the pilot PC"): the pilot PC holds the only
credential; the Worker holds only the verifier. The credential lives in
the pilot PC's off-repo ops config, never in this repository, the form
bundle, or any log. Scope: this one Worker's control plane, nothing else.
Rotation: at each pilot-window boundary and immediately on any suspected
exposure or PC compromise (with D3's emergency rotation). Compromise of
the credential discloses traffic metadata and enables destruction of
pending ciphertext - an availability loss, not a confidentiality one;
detection is by reconciliation (below); response is credential rotation
plus re-solicitation.

**Rate limiting - NAT-safe.** Per-source throttling must tolerate the
convention venue: hundreds of legitimate phones behind one NAT source IP.
Limits are therefore sized generously per-IP (venue-scale bursts pass) and
the real abuse backstops are the size cap, the TTL, the billing ceiling,
and the review gate - not per-IP precision and not the approximate count
cap. A throttled submitter can retry; the
deterministic-lockout failure mode (one venue IP exhausted for the day) is
explicitly designed out by the sizing rule in the deploy runbook.

**Capacity and cost bounds - honest about KV consistency (round-2
amendment).** Pilot scale is ~150 expected / 300 max submissions of
single-digit KB each - trivially inside free-tier KV. KV offers no atomic
counter: concurrent POSTs doing read-modify-write lose increments, and
list counts are eventually consistent. The Worker therefore enforces an
APPROXIMATE total-blob cap (list-derived count, refusing POSTs with a
retry-able error when over it) and the claim of a hard cap is withdrawn;
the actual hard bounds are the per-blob size cap, the TTL (storage
self-drains), platform quotas, and the billing ceiling plus alerts
configured on the Cloudflare account per the deploy runbook. Under burst
abuse the approximate cap may overshoot by the consistency window - that
overshoot is bounded KB at pilot payload sizes and cannot become
unbounded spend. Overload degrades to "remote intake temporarily refuses;
in-app entry continues."

**TTL and loss semantics - honest (rounds 1-2 amendments).** Every blob
carries a TTL. Policy bounds fixed here: TTL is at least twice the
committed maximum pull interval and at most the pilot-window length;
concrete values are deploy-runbook configuration inside those bounds.
Service objective: during August pilots, the puller runs at least daily;
during the convention window, at least hourly (D-030 same-day joiners must
appear by the committee meeting). The puller's run report states the
oldest remaining receipt age; an age past half the TTL is a loud warning
(I12) - expiry should never be a surprise. What expiry MEANS: the relay
cannot identify whose ciphertext expired (by design - no contact fields in
cleartext); "re-solicited" therefore means a broadcast ask, not recovery.

**Reconciliation via a receipt ledger - not a counter (rounds 2-3
amendments; the round-1 running-counter design was unimplementable on KV
and is withdrawn).** On each accepted POST the Worker writes TWO KV
entries in a FIXED ORDER with a defined acknowledgement rule: (1) the
ciphertext blob (blob TTL), then (2) the ledger entry under a separate key
prefix - receipt id, size, arrival timestamp, claimed fingerprint; NO
content. The POST is acknowledged ONLY after both writes succeed. The
crash cases are therefore bounded and named: blob-write failure -> no
ledger, no ack, the submitter sees an error and retries (a fresh receipt
id; no orphan state); ledger-write failure after the blob -> no ack (the
submitter retries), and the orphan BLOB is still found because the puller
lists BOTH prefixes, stages any blob-without-ledger normally, and flags it
as an anomaly (an unacknowledged write) in the run report. A missing
ledger entry is thus observable through the blob, and a missing blob is
observable through the ledger - single-sided platform loss of both halves
of the SAME receipt is the accepted residual (stated, not claimed
detectable).

Ledger entry lifetime (a guaranteed observation window, not a
coincidence): blob TTL + the stated KV consistency margin + AT LEAST one
maximum pull interval. Expiry classification only becomes eligible after
blob TTL + consistency margin, which by construction leaves at least one
full pull cadence during which the ledger entry still exists to be
classified.

Classification precedence (what makes the states a partition): LOCAL
durable facts are strongly consistent and take precedence - a receipt with
a local transport record is STAGED; else one with a local delete-journal
entry is DELETED-BY-ME; only for receipts with NO local record are the
eventually consistent KV observations consulted: blob present -> UNPULLED;
blob absent before TTL + margin -> integrity alert (possible relay-side
deletion or a failed-blob-write orphan ledger entry - the ambiguity is
stated and investigated, re-checked once next run before alarming); blob
absent after TTL + margin -> EXPIRED (counted, drives the broadcast
re-solicit). Evaluated in that order the states are disjoint; nothing is
double-counted and no lost-increment failure mode exists.

**Cutoff epoch (round-3 amendment).** An allowlist edit is not an instant.
The rotation cutoff (D3) is COMPLETE only at: config-change time + the
Worker configuration propagation horizon (a stated deploy-runbook value
for Cloudflare's global propagation) + the maximum request duration
(bounding an in-flight POST that passed the old check before the edit).
The final old-key TTL and the destruction clock start from that completed
epoch, never from the moment the edit was requested.

**Logging and metadata discipline.** The Worker logs no bodies, no
ciphertext, and no authorization headers; request metadata logging is
minimized and retention set to the platform minimum. The Pages form
contains no analytics, no third-party requests, and a CSP that allowlists
exactly one connect destination (the relay origin) - which is also part of
the D8 reviewed bundle. Residual, accepted for v0.1.0: the relay operator
(Cloudflare) necessarily observes traffic metadata - counts, sizes,
timing, source addresses. The consent text must not overclaim against
this (flagged to D-023 review).

**Vendor operational acceptance.** The vendor choice is ruled (D-053);
operating it has recorded preconditions in the deploy runbook: account
under the organization's control with 2FA, recovery factors documented in
the ops log, service terms reviewed at deploy, quotas and billing ceiling
configured, and end-of-window verification that KV holds zero blobs
(post-sweep check). Account suspension mid-window degrades to in-app-only
intake - the convention plan never depends solely on the relay.

**Abuse tolerance via the review gate (D-053).** Anyone can POST garbage
ciphertext or well-formed spam. Tolerated by design: nothing enters the
graph without facilitator approval, so the blast radius of abuse is
facilitator review time plus TTL-bounded, size-capped KV storage under
the platform quota and billing ceiling (the count cap is approximate,
D6) - not graph integrity. Hostile payload CONTENT is neutralized at
render time by D4's validation-and-escape rules, so the review gate itself
is not an XSS vector.

### D7. Ownership-at-approval: approved submissions land unowned (D-056.2)

Approved remote submissions land as UNOWNED, facilitator-created entities.
The facilitator appears as `responsible_human`; no ownership relationship is
bound to the submitter. This keeps the authority matrix unchanged: no new
authority class, no owner-only rights attach to remote records in v0.1.0.
Owner-binding a record to its submitter (the personal-mode direction, R4) is
explicitly deferred; doing it later is an authority-matrix change and
triggers its own adversarial round (D-056.2).

### D8. Browser trust model for the static form (round-1 amendment - new)

The classic hole in encrypt-in-the-browser is the page itself: whoever can
alter the served HTML/JS can swap the key, exfiltrate plaintext alongside
correct encryption, fake the crypto, or serve different bundles to
different viewers. Static hosting cannot eliminate this; the ADR's job is a
concrete, bounded trust model with the residual stated.

**Deterministic, dependency-closed build.** The form is a self-contained
static artifact: vendored pinned libsodium (D3), no runtime code fetches,
no analytics, no third-party requests, inline or same-origin assets only,
and a CSP allowlisting exactly one connect destination (the relay). The
build is reproducible from a reviewed commit: same source -> same bytes.
The build emits a deploy manifest beside the artifact: bundle hash
(SHA-256 over the complete deployable file set), embedded public-key
fingerprint, form_version, consent_text_digest, source commit SHA, and the
vendored libsodium identity (version + package integrity hash).

**Full-bundle pin - executable measurement procedure (round-2
amendment).** The measurement is defined so it can actually be computed
and compared:

- *Canonical manifest - exact grammar (round-3 amendment).* The
  reproducible build, run LOCALLY from the reviewed commit, emits a
  manifest listing every deployable file. Path grammar: UTF-8, NFC
  normalized, forward slashes only, relative to the deploy root, no `.`
  or `..` segments, no backslashes, no percent-encoding; entries sorted
  by byte-wise comparison of the path. Each entry: path, byte length,
  SHA-256 over the file's exact bytes. Serialization: JSON, sorted keys,
  LF newlines, UTF-8 without BOM. The manifest is NOT part of its own
  file set; the manifest hash is SHA-256 over the manifest FILE's exact
  stored bytes (not reconstructed fields). A copy of the manifest MAY be
  deployed beside the bundle for human inspection, but it is
  NON-AUTHORITATIVE: no verifier ever reads the served manifest as an
  authority - the locally pinned copy is the only authority, and the
  served copy is itself just another file to hash-check if listed.
- *Pin provenance.* The pin is the LOCALLY BUILT manifest, recorded
  off-origin in the pilot PC ops config and ops log at deploy time
  (deploy provenance: commit SHA, deployer, time, manifest hash). The pin
  is never copied from the deployed origin - fetching the file list or
  expected hashes from the served origin would recreate the same-origin
  trust hole this section exists to close.
- *Deploy.* Exactly the built file set is deployed - nothing more; the
  CSP and the dependency-closed build mean the verified HTML references
  no file outside the manifest.
- *Verification.* The puller (and the post-deploy check) fetches EVERY
  path listed in the PINNED manifest from the deployed origin: cache
  bypassed (no-store request cache mode plus a cache-busting request
  where the platform honors it - a cached response proves an old
  representation, not what the origin serves now); redirects refused
  (final URL must equal the requested same-origin URL); status must be
  200; content-encoding decoded to identity bytes before hashing. Each
  file's bytes-hash and length are compared against the pin; any
  mismatch, missing file, redirect, or non-200 halts the line exactly
  like a key mismatch.
- *Service workers and client state (round-3 amendment).* The form
  registers NO service worker, and the deploy runbook forbids ever
  introducing one on this origin. A native puller does not execute
  service workers, so its verification measures served bytes - it cannot
  see a previously installed hostile service worker controlling an
  attendee's browser, nor other persistent client state. That is folded
  into the accepted persistent-client/split-view residual below: the
  manifest + CSP measure the SERVED artifact, not all executable client
  state, and the text claims no more than that. Stated residual: a
  static host does not enumerate its files, so an EXTRA planted file is
  not detectable by fetch - but it is unreferenced by the verified
  HTML/CSP and so cannot execute in a clean client's page load.
- *Unreachable origin* (offline pilot PC, Pages outage): pulls may
  proceed on the local-key check alone with a loud WARN in the run report
  (already-sealed ciphertext is not endangered by current-bundle state),
  but NO new solicitation (QR presentation) until the deployed bundle
  verifies - the check protects future submitters, and the ceremony
  companion carries the same rule.

**Repo/deploy-chain protections.** Pages deploys from the public repo:
branch protection on the deploying branch, deploys only from reviewed
commits, and the GitHub account custody rules already in force for the
D-060 push credential. The public repo makes the source reviewable by
anyone; the bundle pin is what ties the served artifact to that source.

**Residual risk - stated and accepted for v0.1.0.** The puller's check is
periodic and observes the bundle the PULLER is served. A compromised host
or path that serves the legitimate bundle to the verifier and a malicious
bundle to a targeted attendee (split-view delivery), or that compromises
phones between checks, is not detectable by this design. No static-hosting
design closes this without an out-of-band verified client, which is
overbuilt for v0.1.0 (and "no app install" is a pilot usability
constraint). Accepted residuals, recorded: split-view delivery against
targeted attendees; compromise windows between pin checks. The mitigations
are the short pilot windows, deploy-time and per-run verification, and the
facilitator review gate on everything that enters the graph. Absolute
"nothing readable ever transits" phrasing is accordingly withdrawn in
favor of: content is encrypted on the phone by a form whose integrity is
pinned and verified as above, with the stated residual.

## Options considered and rejected

1. **External form platform (Google/Microsoft Forms)** - rejected by the
   human (D-053): a third party would hold readable PII, violating the prime
   directive; also adds a vendor dependency to the consent instrument.
2. **Routing remote intake through SyncTransport** - rejected: intake is not
   peer synchronization - it is one-way, pre-consent-review, non-op-shaped
   data. Using the seam would prematurely instantiate the human-gated
   network path and turn ADR-002 A-B8's deliberately provisional v0 seam
   into a load-bearing production dependency. Ingest semantics (validation,
   dedup, human review) do not fit op exchange.
3. **TLS-only relay holding plaintext** - rejected: the server would hold
   readable personal data at rest; sealed boxes keep the relay
   ciphertext-only, so relay compromise leaks no content.
4. **Pilot PC listens for direct submissions** - rejected: the graph never
   listens on the network (R5 stance, preserved by D1); an inbound port on
   the facilitator machine at a convention is both a security regression and
   operationally fragile (NAT, uptime, venue networks).
5. **Pending queue inside the op log** (quarantined ops or a "pending"
   OpKind) - rejected: the append-only log would permanently record rejected
   and spam submissions, violating the consent semantics of D-030; raw
   pre-validation payloads do not fit op schema; and unreviewed PII would
   sit inside the structure that export/sync machinery folds over (ADR-002
   A-B5) instead of structurally outside it.
6. **Authenticated intake (accounts, magic links)** - rejected: no
   submitter auth in v0.1.0 or v1.0 is a human ruling (D-053); the
   facilitator review gate is the substitute for submitter authentication.
7. **Wipe-then-write ordering** (delete the relay blob on pull, stage
   after) - rejected: a crash between wipe and durable stage loses a
   consented person's submission irrecoverably. Durable-write-first with
   idempotent re-pull is strictly safer and costs only a delayed delete.
8. **Localized offline/hotspot intake now** - deferred, not rejected: the
   human named it the later stretch goal (D-053); the relay path is the
   v0.1.0 commitment.
9. **A second signing ceremony / signed deploy manifest as the bundle trust
   root** (round 1) - rejected: a signature verified by code served from
   the same origin adds ceremony without an independent trust root; the
   off-origin full-bundle hash pin held on the pilot PC provides the same
   binding with less machinery. Revisit only if a genuinely independent
   verification channel (e.g. a verified client) enters scope post-v0.1.0.
10. **Encrypted queue backup** (round 1) - rejected for v0.1.0: a second
    copy of decrypted submissions is a privacy regression to buy
    durability that re-solicitation already bounds; single-disk loss
    between pull and approval is accepted and recorded in D4.
11. **A database or distributed queue** (round 1, right-sizing) - rejected:
    at 150-300 records, an immutable-file queue with atomic renames, a
    lock file, and checksums meets the durability contract with radically
    less surface.

## Consequences

Positive:

- No server in the system ever holds readable personal data; the only
  machine that decrypts in operation is the pilot PC. Relay loss or
  compromise loses nothing readable.
- The consent instrument (the form) and the facilitator gate are the only
  door into the graph; abuse of the open endpoint cannot touch graph
  integrity, and hostile content is neutralized at render time.
- R5's core stance survives intact: the SyncTransport seam is untouched, the
  graph still never listens, the amendment is a single named inbound ingest
  path, and the D1 module fence keeps networking out of every core crate.
- The queue's placement outside the op log AND outside the worktree keeps
  unreviewed data structurally away from both graph history and git.
- The approval transaction's preassigned ids and digests, over the new
  idempotent durable batch-append seam, make facilitator approval
  crash-safe and duplicate-free AT THE DURABLE-LOG LEVEL - not merely at
  fold level (round-2 correction; the seam is an additive cn-store API
  the implementation must land first).

Negative / accepted:

- The facilitator private key remains a single point of total LOSS for
  in-flight submissions (mitigated by ceremony backups), and - stated
  plainly after round 1 - a leaked key retroactively exposes captured
  ciphertext (no forward secrecy), and a full pilot-PC compromise exposes
  the decrypted queue and credentials, not just staged submissions.
- Single-disk loss of the queue between pull and approval loses those
  submissions; accepted in exchange for not multiplying PII copies.
- The Pages form is gate-coupled to the repo's publish preconditions and
  now also to the D8 deploy-provenance discipline (reviewed commit, bundle
  pin, post-deploy verification) - deploys are ceremonies, not pushes.
- Split-view delivery against targeted attendees by a compromised static
  host is not detectable by this design; accepted for v0.1.0 with short
  windows and per-run verification as the bounds.
- Traffic metadata at the relay operator is visible (counts, sizes, timing,
  source addresses); accepted for v0.1.0; the consent text must not
  overclaim against it.
- No submitter auth means dedup rests on transport/semantic hashes plus
  facilitator near-duplicate review; facilitator review load scales with
  abuse volume (bounded in practice by the TTL, the approximate count
  cap, and the platform quota/billing ceiling - not by a hard count
  guarantee).
- New verification surface: the puller, both envelope formats, the queue
  record/sidecar formats, the crash/recovery protocol, and the pii-scan
  tripwires all need tests (including fault-injection at each protocol
  boundary and positive tripwire fixtures) - additions to check-all.
- Two new formats enter the I7 version-discipline set (intake envelope,
  queue record) and must be maintained alongside the existing ones.

## Consent-text implications flagged to D-023 review

Round 1 found the current consent draft overclaims against this ADR in four
places; the D-023 solo pass should reconcile (tracked in the draft doc):

1. "Nothing else is collected" - the relay operator necessarily observes
   traffic metadata (D6); wording should scope the claim to form content.
2. "Only the facilitator's computer holds the key" - offline recovery
   copies exist (D3); wording should say the key is used only on the
   facilitator's computer.
3. Removal promise ("your information will be taken out of the network") -
   the op log is append-only (ADR-002); the human must decide whether
   removal means no-longer-projected (current architecture) or erasure
   (which would need its own design), and word the promise accordingly.
4. The confirmation screen must distinguish "the relay accepted your sealed
   envelope" from "the facilitator has it" (D6 TTL semantics).

## Open questions (deferred to implementation - none block acceptance)

Resolved into the decision by the round-1 and round-2 amendments: libsodium
distribution and bundle integrity incl. the canonical measurement (D3/D8);
pull/delete credential contract (D6); TTL and cadence policy plus loss
semantics and the receipt-ledger reconciliation (D6 - concrete values
remain deploy runbook configuration inside the fixed bounds); receipt
confirmation semantics (D6; exact wording is D-023's); rotation/drain with
relay admission cutoff (D3/D6); the durable batch-append seam and crash
recovery table (D4); sidecar binding and the approved-record closeout
(D4/D5). Remaining, genuinely deferrable:

- Puller packaging: `cn intake` subcommand (leaning yes) vs separate small
  tool - the D1 module fence and the D5 actor identity hold either way.
- Near-duplicate similarity heuristic tuning at pilot scale - the fixed
  constraints (transparent reasons, conservative matching, no auto-merge)
  are in D4; thresholds are implementation.
- Rust-side sealed-box binding choice - deploy-gated by cross-implementation
  test vectors (JS-sealed opened by Rust puller), malformed-ciphertext
  rejection tests, and fail-closed behavior, all mandatory in check-all
  before the deploy bar.
- Pin-config file layout in the off-repo ops directory - constraints fixed
  (off-repo, ACL'd, atomically updated, versioned per I7); layout is the
  keygen ceremony runbook's.
- One keypair or two across pilot/convention windows - operational choice
  under the D3 rotation protocol.
- ~~Queue retention policy for rejected records~~ RESOLVED by the human
  (D-059.11): kept in full for the pilot window, then purged in one
  recorded sweep; D4 carries the ruling, extended to pending and approved
  records' lifecycle at window close.
