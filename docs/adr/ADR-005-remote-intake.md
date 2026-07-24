# ADR-005: Remote Intake - Sealed-Envelope Relay and Facilitator Pending-Review Queue

- Status: DRAFT - round 1 FAIL judged and amended 2026-07-24; pending round 2
- Date: 2026-07-24 (amended same day after adversarial round 1)
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
  key-custody claims. All five judged valid and addressed by this amendment.

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

**Rotation protocol (drain-before-flip).** Rotation is never a simple form
flip, because cached pages and open tabs can produce old-key ciphertext
after a redeploy:

1. Pause distribution: stop presenting the QR / announce the pause window.
2. Drain: pull and reconcile the relay until receipt counts match POSTs
   observed for the window (D6 reconciliation) or relay TTL has elapsed
   since pause.
3. Run the new keygen ceremony; deploy the new form; independently verify
   the deployed bundle hash and embedded key (D8 post-deploy check).
4. Retain old-key decrypt capability for the maximum old-form lifetime:
   relay TTL plus a cache horizon (Pages/CDN cache lifetime plus an
   open-tab allowance, fixed in the deploy runbook). Envelopes arriving
   sealed to the old key during this window still decrypt; the queue
   records `key_used`.
5. Destroy the old key and its backups only after reconciliation shows no
   old-key envelope can still be in flight.

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
(`pending | approved | rejected`), the full decision history (array of
`{state, reviewer, decided_at, reason}` - never overwritten, only appended
to), reviewer notes, validation-report reference, near-duplicate candidates
surfaced and the facilitator's disposition, and - once approved - the
approval transaction fields below. Sidecar updates use the atomic-replace
protocol below, and the history array means an overwritten sidecar still
carries every prior decision.

**Windows crash-state protocol (replaces round-0's four verbs).** All queue
writes follow one primitive: write to a temp file in the queue directory
(exclusive create, unique name), flush to disk (`FlushFileBuffers` via the
runtime's fsync), close, then atomically rename onto the final path
(`MoveFileEx` with `REPLACE_EXISTING | WRITE_THROUGH` semantics), then
flush the directory handle where the platform allows. Every record and
sidecar carries an internal checksum; a reader that finds a temp file, a
missing sidecar, or a checksum mismatch treats it as a torn write and
follows the recovery rules below. A single-instance lock file at the queue
root (exclusive create; stale-lock takeover only with process-liveness
check) forbids concurrent pullers/review processes - two instances is a
refuse-to-run, not a race.

Per-receipt staging sequence: stage payload record via the primitive ->
stage `pending` sidecar via the primitive -> re-read and checksum-verify
both -> only THEN issue the relay delete for that receipt. A crash anywhere
before the delete leaves the receipt on the relay; the next run re-pulls it
and transport dedup (below) makes re-staging a recorded no-op. A delete
whose acknowledgement is lost is retried; deleting an already-deleted
receipt is idempotent at the relay (D6).

**Approval transaction (queue -> op log).** Approval is a small
write-ahead protocol, because the graph write and the queue update cannot
crash-atomically happen together:

1. Facilitator approves in the review UI. BEFORE any graph write, the
   sidecar is updated to `review_state: approved-intent` with the complete
   approval batch: preassigned op ids (UUIDv7, generated now), a
   `batch_digest` over the ops to be submitted, and the reviewer/timestamp
   entry in the decision history.
2. The ops are submitted through the normal authorized path (cn-api submit
   with the facilitator viewer) USING those preassigned op ids.
3. On submit success, the sidecar is updated to `approved` with the
   submit report reference.

Recovery: a sidecar found in `approved-intent` at startup means a crash in
step 2-3. The recovery scan asks the store which of the preassigned op ids
already exist (op-id dedup is ADR-002's existing contract): all present ->
complete to `approved`; none present -> resubmit the same batch with the
SAME ids (idempotent); partial -> resubmit the full batch with the same
ids, the store's per-op dedup makes already-applied ops no-ops. Duplicate
entities from retried approvals are impossible because ids are preassigned
and reused, never regenerated. Rejection is a single sidecar update
(`rejected` plus history entry); records then persist under D-059.11 until
the recorded purge sweep.

**Dedup - transport and semantic layers (round-1 amendment).**
`submission_id` is client-controlled and therefore cannot be the only key:

- *Transport dedup* (re-pull safety): key `(receipt_id, ciphertext_hash)`.
  A re-pulled receipt matching an existing record is a recorded no-op
  (I12).
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

**Retention and lifecycle - all three states.** Rejected: per D-059.11,
kept in full for the pilot window, then purged in ONE RECORDED SWEEP at
window close. Pending at window close: reviewed to a decision or purged in
the same sweep - the queue is never left holding undecided PII after the
window. Approved: the queue record is the audit trail linking consent to
ops; it is retained for the pilot window and included in the same
close-of-window sweep decision (purge or archive is decided at the sweep
and recorded). The sweep produces a dated, non-PII purge manifest - counts,
record ids, hashes, sweep operator - which is the D-059.11 "recorded"
artifact and lives in the facilitator ops log (off-repo), with a one-line
DECISIONS.md entry referencing it.

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

### D5. Intake provenance envelope (I6) - with trust status

Every entity and edge produced by an approved remote submission carries the
full provenance envelope. Round-1 amendment: values are recorded WITH their
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
- The queue `record_id`: the durable link from graph ops back to the full
  queue audit record (which holds the decision history and hashes).

Tier: every pilot entry enters at T1; the tier authority is ATNI Climate
(D-034). Per-field tier UX is post-pilot work.

### D6. The relay: API security contract and threat model

**API contract (round-1 amendment - previously implicit).**

- `POST /submit` (public, no auth): accepts one outer envelope within the
  size cap. The Worker GENERATES the receipt id (128-bit random,
  server-side; the client can neither choose nor predict KV keys, so one
  POST can never overwrite another). Response: the receipt id and nothing
  else. The acknowledgement MEANS "the relay stored your sealed envelope";
  it does not mean staged, reviewed, or approved - the D-023 confirmation
  wording must say so (see TTL semantics below).
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
the real abuse backstops are the size cap, total-capacity cap, and the
review gate - not per-IP precision. A throttled submitter can retry; the
deterministic-lockout failure mode (one venue IP exhausted for the day) is
explicitly designed out by the sizing rule in the deploy runbook.

**Capacity and cost bounds.** Pilot scale is ~150 expected / 300 max
submissions of single-digit KB each - trivially inside free-tier KV. The
Worker enforces: per-blob size cap, a total-stored-blobs cap (an order of
magnitude above pilot scale; POSTs beyond it are refused with a retry-able
error rather than allowing unbounded storage), and the Cloudflare account
carries a billing alert plus a spending ceiling per the deploy runbook.
Overload therefore degrades to "remote intake temporarily refuses; in-app
entry continues" - it cannot become spend or a crowd-out of the queue's
review capacity beyond the cap.

**TTL and loss semantics - honest (round-1 amendment).** Every blob
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
Reconciliation: the Worker maintains a running POST counter (a count, no
content); the puller compares receipts pulled + deleted + expired against
it each run and investigates gaps - this is also the detection path for
relay-side deletion, truncation, or withholding (integrity failures that
ciphertext-only design does not otherwise surface).

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
facilitator review time plus TTL-bounded, size-capped, count-capped KV
storage - not graph integrity. Hostile payload CONTENT is neutralized at
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

**Full-bundle pin - not key-only (round-1 amendment).** At deploy time the
facilitator records the bundle hash and manifest in the off-repo ops log
(deploy provenance: which commit, who deployed, when, what hash). The
puller's pre-run check (D3) verifies the DEPLOYED bundle hash against this
off-repo pin - extending the ceremony's key-only pin to the whole
executable artifact. A served bundle whose hash mismatches the pin halts
the line exactly like a key mismatch. A hash or manifest served by the
compromised origin itself proves nothing; the pin's authority is that it
is held off-origin, on the pilot PC, recorded at a deploy the facilitator
performed.

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
- The approval transaction's preassigned-op-id protocol makes facilitator
  approval crash-safe and duplicate-free on top of ADR-002's existing op-id
  dedup.

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
  abuse volume (bounded by the capacity cap).
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

Resolved into the decision by the round-1 amendment: libsodium distribution
and bundle integrity (D3/D8); pull/delete credential contract (D6); TTL and
cadence policy plus loss semantics (D6 - concrete values remain deploy
runbook configuration inside the fixed bounds); receipt confirmation
semantics (D6; exact wording is D-023's); bundle-verification model (D8);
rotation/drain protocol (D3). Remaining, genuinely deferrable:

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
