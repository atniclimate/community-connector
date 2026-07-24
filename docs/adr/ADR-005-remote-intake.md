# ADR-005: Remote Intake - Sealed-Envelope Relay and Facilitator Pending-Review Queue

- Status: DRAFT - pending adversarial round
- Date: 2026-07-24
- Phase: 3 (intake pipeline P3.5/P3.6 plus the D-053 relay; deploy gated on the
  D-055 publish preconditions)
- Drivers: R3 (data entry and ingestion), R5 as amended by D-053 (one remote
  intake path), R10/I6 (provenance envelope on everything), I1 (no PII in the
  repo or any prompt), I7 (versioned persisted formats, unknown-major
  rejection), I12 (validation reports); rulings D-030 (the intake form is the
  individual consent instrument; form respondents only), D-034 (all pilot data
  enters at T1), D-050 (August internal pilots, convention 2026-09-14), D-053
  (intake architecture and conditionally opened gates), D-056.1 (required
  scope of this ADR), D-056.2 (ownership-at-approval), D-056.4 (dedup and
  near-duplicate surfacing, risk register)

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
- No authentication in v0.1.0 or v1.0 (D-053) - the endpoint is public.
- The graph is local-first and never listens on the network; R5's stance is
  "network-ready, not networked", and choosing protocols or identity
  standards is human-gated.

The human resolved the resulting tension in the 2026-07-24 gate-grill session
(D-053) by choosing a sealed-envelope relay: the only server in the path
stores ciphertext it cannot read, and the only machine that can decrypt is
the facilitator's pilot PC. The human conditionally opened three gates for
exactly this path: the public remote for the GitHub Pages form, the
Cloudflare Workers hosting vendor, and the associated spend - with
preconditions (license in-repo, D-055 pre-publish sweep passed, core
stability) before any push or deploy.

D-053 trips the architecture-stance-change rule three ways: it amends R5, it
introduces a new persistent store (the pending-review queue), and it
introduces new persisted formats. D-056.1 therefore requires this ADR plus
one adversarial round, and fixes its scope. This document is that ADR.

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
  index, or transform payloads; its API is store-blob and
  fetch-then-delete-blob.
- The R5 amendment is exactly this narrow: one pull-based ciphertext relay
  for intake. Peer sync, federation, identity standards, and every other
  networking decision remain human-gated and untouched by this ADR.

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
  the public repo. Pages is interface only: it holds no secrets, runs no
  backend, and no readable submission data ever rests there or transits it -
  encryption happens in the browser before the payload leaves the phone.
- The browser seals the submission to the facilitator public key (D3) and
  POSTs only ciphertext to the relay.
- The relay is a minimal Cloudflare Worker writing to KV: store the blob,
  return a receipt id, expire unpulled blobs by TTL (D6).
- The pilot PC puller fetches ciphertext, decrypts locally, writes the
  submission durably into the pending-review queue (D4), verifies the write,
  and only then deletes that receipt from the relay.
- Fallback when the relay or form is unavailable: direct in-app facilitator
  entry, which is the primary intake path anyway (D-053).

### D3. Sealed-envelope payload format, versioning, and key custody (I7)

**Cryptography.** Submissions are libsodium sealed boxes
(`crypto_box_seal`) to the facilitator's X25519 public key: the sender uses
an ephemeral key pair, so only the holder of the facilitator private key can
decrypt, and submissions are unlinkable to sender keys (the form itself is
the consent instrument, so sender anonymity at the crypto layer is
acceptable). The libsodium JS build is vendored into the static form bundle
at a pinned version - no CDN, no runtime fetch of crypto code.

**Key custody.** The facilitator private key exists ONLY on the pilot PC. It
is generated in the facilitator keygen ceremony (D-056.3/D-056.4): offline
generation, an offline backup held by the facilitator (key backup is not
repo data), and the key never appears in the repo, the relay, the Pages
bundle, any commit, or any Codex prompt. The public key and its fingerprint
are embedded in the static form and are the only key material that is
world-readable.

**Envelope format.** Two layers, both explicitly versioned (I7):

- Outer envelope (cleartext to the relay, deliberately minimal):
  `intake_envelope_version` (semver), `recipient_key_fingerprint`, and the
  sealed-box ciphertext (base64). Nothing else; the relay learns format
  version, target key, size, and timing - no content.
- Inner payload (plaintext only after local decryption on the pilot PC):
  `submission_version` (semver), `submission_id` (client-generated UUID, the
  dedup key), `form_version`, `captured_at` (client capture timestamp), and
  the form fields.

The puller rejects unknown MAJOR versions of either layer loudly and records
them in the validation report; unknown MINOR fields are ignore-and-preserve
(the ADR-002 version policy, applied to intake formats).

**Key pinning.** The puller pins the facilitator key fingerprint: before a
pilot run it verifies that the fingerprint published in the deployed form
matches the fingerprint of the local private key, and a mismatch stops the
line. The `recipient_key_fingerprint` on each envelope lets the puller
distinguish "sealed to my current key", "sealed to a retired key I still
hold", and "sealed to a key I do not have" (the last is a loud typed error,
never a silent skip - I3).

**Rotation story.** Rotation is: run a new keygen ceremony -> update the
form's embedded public key and fingerprint -> redeploy Pages -> the puller
retains the retired private key for a drain window bounded by the relay KV
TTL (so in-flight envelopes sealed to the old key still decrypt) -> retire
and destroy the old key after the window. On suspected compromise of the
pilot PC or private key: rotate immediately; the relay held only ciphertext,
so exposure is bounded to submissions already pulled and staged locally.

### D4. Pending-review queue: outside the op log, durable-write before relay-wipe

**Placement.** The queue is NOT part of the cn-store op log, and pending
submissions produce no operations. Rationale: the op log is append-only
domain history; rejected submissions must leave no trace in graph history
(D-030 consent semantics - a person the facilitator rejects, or spam, must
never become permanent log content); queue records carry raw, pre-validation,
pre-consent-confirmation data that has no entity ids and no place in op
schema; and keeping unreviewed PII out of the op log keeps it structurally
unreachable by export and sync machinery (ADR-002 A-B5). Only
facilitator-approved submissions generate operations, through the normal
authorized submit path.

**Persisted format (I7).** The queue is a directory of one JSON record per
submission, each carrying `queue_record_version` (semver). Readers reject
unknown MAJOR versions loudly; unknown MINOR fields are ignore-and-preserve.
A record holds: the outer-envelope metadata, the decrypted inner payload,
the relay receipt id, the pull timestamp, the review state
(`pending | approved | rejected`), reviewer notes, and - once approved - the
op ids the approval produced (audit linkage).

**Durability ordering.** The puller's per-receipt sequence is: write the
queue record, fsync, read it back and verify it parses, and only THEN issue
the relay delete for that receipt. Relay-wipe is per-receipt and strictly
after that receipt's record is durable. A crash anywhere in the sequence
results at worst in a re-pull of an undeleted receipt, and re-pull is
idempotent: `submission_id` dedup makes the second staging a recorded no-op.

**Staging location and PII containment.** The queue lives outside the
repository, or in a repo-local staging directory that is gitignored - never
committed (I1; real-data gate process). Two enforcement layers: the
gitignore entry, and pii-scan coverage - the scan treats any queue-format
record (matched by its version marker) found in tracked or staged content as
a blocking finding, so a mis-placed queue file cannot pass the pre-commit
hook.

**Dedup and near-duplicate surfacing (D-056.4).** Exact duplicates are
dropped by `submission_id` (recorded, not silent - I12). Because there is no
auth, the same person can submit twice with different UUIDs; the review UI
therefore surfaces near-duplicate candidates (normalized-name plus
affiliation similarity against both the queue and the existing graph) to the
facilitator at review time. The facilitator decides approve / reject /
merge-by-hand; tooling never auto-merges.

### D5. Intake provenance envelope (I6)

Every entity and edge produced by an approved remote submission carries the
full provenance envelope, populated as follows:

- `actor`: the intake tooling, as a software-agent identifier with its
  version (the puller/ingest component, not the facilitator).
- `responsible_human`: the reviewing facilitator - required because the
  actor is non-human (ADR-002 D2), and correct because the facilitator's
  approval is the act that admits the data.
- Capture timestamp: `captured_at` from the inner payload (when the person
  filled the form), carried alongside the approval timestamp.
- `form_version`: the form the submitter actually saw - this is what makes a
  submission interpretable against the consent text that was displayed.
- Relay receipt id: the hop through the relay is a custody-relevant event.
- Client-generated submission UUID: the dedup key, recorded so re-ingest and
  same-day re-pulls stay idempotent (the D-030 fast-re-ingest requirement).

Tier: every pilot entry enters at T1; the tier authority is ATNI Climate
(D-034). Per-field tier UX is post-pilot work.

### D6. Threat model for the unauthenticated endpoint

The relay accepts anonymous POSTs by design (no auth, D-053). Controls and
accepted residual risk:

- **Payload size cap.** A sealed form submission is small (single-digit KB).
  The Worker rejects oversized bodies before any KV write, capping per-blob
  storage abuse.
- **Rate limits.** Per-source rate limiting at the Worker, with Cloudflare's
  platform-level protections behind it. The failure mode of throttling a
  burst is acceptable: in-app facilitator entry is the primary path, and a
  legitimate submitter can retry.
- **KV TTL.** Every blob carries a TTL sized to the pull cadence of the
  active pilot window (days, not months). The relay is a buffer, not a
  store: if the puller dies, unpulled ciphertext expires rather than
  accumulating; a lapsed submission is re-solicited, not recovered.
- **No public read.** The public surface is write-only. Pull and delete
  require a credential held only by the pilot PC and configured directly in
  the Worker environment - never in this repository or the form bundle.
- **Abuse tolerance via the review gate (D-053).** Anyone can POST garbage
  ciphertext or well-formed spam. Tolerated by design: nothing enters the
  graph without facilitator approval, so the blast radius of abuse is
  facilitator review time plus TTL-bounded, size-capped KV storage - not
  graph integrity.
- **Compromise of the relay.** Yields ciphertext plus traffic metadata
  (submission counts, sizes, timing, source addresses visible to the
  operator). No readable personal data. This metadata exposure is accepted
  for v0.1.0 and recorded here as a known limitation.
- **Denial of service.** Acceptable failure mode: remote intake goes down,
  in-app entry continues. The convention plan never depends solely on the
  relay.

### D7. Ownership-at-approval: approved submissions land unowned (D-056.2)

Approved remote submissions land as UNOWNED, facilitator-created entities.
The facilitator appears as `responsible_human`; no ownership relationship is
bound to the submitter. This keeps the authority matrix unchanged: no new
authority class, no owner-only rights attach to remote records in v0.1.0.
Owner-binding a record to its submitter (the personal-mode direction, R4) is
explicitly deferred; doing it later is an authority-matrix change and
triggers its own adversarial round (D-056.2).

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
6. **Authenticated intake (accounts, magic links)** - rejected: no auth in
   v0.1.0 or v1.0 is a human ruling (D-053); the facilitator review gate is
   the substitute for submitter authentication.
7. **Wipe-then-write ordering** (delete the relay blob on pull, stage
   after) - rejected: a crash between wipe and durable stage loses a
   consented person's submission irrecoverably. Durable-write-first with
   idempotent re-pull is strictly safer and costs only a delayed delete.
8. **Localized offline/hotspot intake now** - deferred, not rejected: the
   human named it the later stretch goal (D-053); the relay path is the
   v0.1.0 commitment.

## Consequences

Positive:

- No server in the system ever holds readable personal data; the only
  decrypt-capable machine is the pilot PC. Relay loss or compromise loses
  nothing readable.
- The consent instrument (the form) and the facilitator gate are the only
  door into the graph; abuse of the open endpoint cannot touch graph
  integrity.
- R5's core stance survives intact: the SyncTransport seam is untouched, the
  graph still never listens, and the amendment is a single named inbound
  ingest path.
- The queue's placement outside the op log keeps unreviewed data
  structurally unreachable by export and sync machinery.

Negative / accepted:

- The facilitator private key is a single point of total loss for in-flight
  submissions (D-056.4 risk register). Mitigated - not eliminated - by the
  keygen ceremony, the offline backup, and fingerprint pinning in the
  puller.
- The Pages form is gate-coupled to the repo's publish preconditions, which
  makes the D-055 pre-publish sweep pilot-critical-path.
- Traffic metadata at the relay operator is visible (counts, sizes, timing,
  source addresses); accepted for v0.1.0.
- No auth means dedup rests on the submission UUID plus facilitator
  near-duplicate review; facilitator review load scales with abuse volume.
- New verification surface: the puller, the envelope formats, and the queue
  format need tests, and pii-scan gains queue-format detection - additions
  to the check-all battery.
- Two new formats enter the I7 version-discipline set (intake envelope,
  queue record) and must be maintained alongside the existing ones.

## Open questions (for the adversarial round and implementation)

- Exact libsodium JS distribution and how the vendored bundle's integrity is
  pinned in the static form (build-time hash vs subresource integrity).
- Pull/delete credential mechanics on the Worker (header shape, rotation
  cadence) - configured in the Worker environment, never in-repo; the
  mechanism still needs a written runbook.
- Concrete KV TTL and pull cadence for the August pilots versus convention
  day (same-day joiners must appear by the committee meeting, D-030).
- Whether the puller ships as a `cn` CLI subcommand (leaning yes, for
  provenance and code reuse with cn-ingest) or as a separate small tool.
- Near-duplicate similarity heuristic scope at pilot scale (~150 expected,
  300 max signups, D-052) - how much fuzziness before facilitator review
  drowns in candidates.
- Whether the Worker returns the receipt id to the phone as a user-facing
  "submitted" confirmation, and what that wording needs from D-023 human
  review.
- ~~Queue retention policy for rejected records~~ RESOLVED by the human
  (D-059, 2026-07-24): rejected records are kept IN FULL in the gitignored
  queue for the duration of the pilot window (audit and un-reject ability),
  then purged in one recorded sweep. The purge sweep is a mandatory dated
  checklist item at pilot-window close; the accepted implication - declined
  people's decrypted data persists on the pilot PC for the window - is
  recorded with the ruling. D4's queue format carries the ruling unchanged.
