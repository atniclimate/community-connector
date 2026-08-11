# Blueprint: Remote Intake Relay - Pages Form, Worker Relay, and Native Puller

Status: director blueprint, 2026-08-11. Implements the remote half of
the D-053 intake architecture per ADR-005 (ACCEPTED D-068, eight adversarial
rounds). Permission-adjacent at the sealed-box boundary and the relay
control plane: implementation gets a MANDATORY adversarial round before
its commits are accepted. The in-app half (entry forms, wizard, queue
formats, approval transaction) is LANDED and ACCEPTED (D-076..D-080,
five adversarial rounds); this blueprint builds the components that feed
it remotely.

Dependency: the intake-pipeline blueprint's queue formats, cn-ingest
record/sidecar model, cn-store durable seam, `cn intake apply`, and the
app's FSA adapter are the contract this work plugs into. The puller
writes the same `QueueRecord` with `SubmissionSource::Remote`; the
facilitator wizard's review/approval flow needs ZERO changes to serve
remote records.

Deploy gate: D-059.8 remains. Building and testing against synthetic data
is UNLOCKED; deploying the Pages form and Worker relay requires: this
pipeline working + keygen ceremony executed + D-023 sign-off on form
text. The blueprint stops at "tested locally with synthetic data" and
does not cross the deploy bar.

## Design intent

The remote intake relay is the QR-code path for convention attendees
(D-030, D-050): scan QR -> fill form on phone -> browser seals to
facilitator public key -> POST ciphertext to relay -> pilot PC pulls,
decrypts, stages -> facilitator reviews in the wizard -> `cn intake apply`
admits to the graph. The relay stores only ciphertext it cannot read;
the only machine that decrypts is the pilot PC. Controlling principles:

- **ADR-005 is binding.** This blueprint implements, never reinterprets.
  Every D-section reference below is normative; deviations are bugs.
- **Module fence (D1).** All network I/O lives in ONE adapter at the
  CLI/tooling edge (`cn intake pull`). No HTTP client, socket, or network
  dependency enters cn-model, cn-schema, cn-store, cn-perm, cn-graph,
  cn-api, cn-wasm, or cn-ingest.
- **Crypto binding is cross-implementation verified.** The browser seals
  with libsodium.js; the puller opens with a Rust implementation. Both
  directions get test-vector coverage before any code ships.
- **The form is a build artifact, not hand-written HTML.** A reproducible
  build from a reviewed commit produces the deployable file set and its
  canonical manifest (D8).

## 1. Sealed-box crypto binding (Rust, cn-ingest)

ADR-005 D3 specifies libsodium sealed boxes (`crypto_box_seal`). The
browser side vendors `libsodium-wrappers` (pinned exact version); the
Rust side needs a compatible implementation.

**Binding choice.** Evaluated against the hard requirement (D3, ceremony
design section 3): cross-implementation test vectors - a fixture of
ciphertexts produced by libsodium.js MUST open in the Rust
implementation, and vice versa.

Candidates (evaluated at implementation time; this blueprint fixes the
evaluation criteria, not the winner):
- `crypto_box` (RustCrypto) with sealed-box support - pure Rust, well
  maintained, no C dependency.
- `dryoc` - libsodium-compatible Rust API, pure Rust default with
  optional libsodium-sys backend.
- `libsodium-sys` (sodiumoxide successor bindings) - wraps the C
  reference implementation; interop guaranteed but adds a C build
  dependency.

Selection criteria in priority order: (1) sealed-box interop with
libsodium.js proven by test vectors; (2) pure Rust (no C build
dependency in the workspace); (3) maintained, audited, or widely used;
(4) minimal API surface (we need keygen, seal, open, nothing else).

**What lands in cn-ingest.** A thin `crypto` module (or
`sealed_box` module) exposing:

```rust
pub struct Keypair { pub public: PublicKey, pub secret: SecretKey }
pub struct PublicKey([u8; 32]);
pub struct SecretKey([u8; 32]);
pub struct Fingerprint([u8; 16]);

pub fn generate_keypair() -> Keypair;
pub fn fingerprint(public: &PublicKey) -> Fingerprint;
pub fn seal(plaintext: &[u8], recipient: &PublicKey) -> Vec<u8>;
pub fn open(ciphertext: &[u8], recipient: &Keypair) -> Result<Vec<u8>, CryptoError>;
```

- `Fingerprint`: BLAKE2b-256 over the raw 32-byte public key, truncated
  to 16 bytes, rendered as 8 lowercase hex groups of 4 (ceremony design
  section 2). Display impl produces the `3f9a-1c02-...` format; FromStr
  parses it back.
- `SecretKey` implements `Zeroize` (zeroize crate, already a transitive
  dep of most crypto crates) so the decrypted key does not linger in
  memory after the puller finishes.
- `CryptoError` is a variant of `IngestError` (no content in the error
  message - a failed open leaks nothing about why).

**Key file I/O.** The ceremony design section 2 specifies JSON envelopes
with `format: "cn-intake-key"`, `schema_version`, `role`, `created_at`,
`fingerprint`. Two file types:

- `public.json`: role `public`, contains the raw public key (hex).
- `secret.json`: role `secret-encrypted`, contains XSalsa20-Poly1305
  secretbox of the raw secret key under an Argon2id-derived key from
  the facilitator passphrase (libsodium `crypto_secretbox` defaults).
  The nonce and Argon2id parameters (opslimit, memlimit, salt) are
  stored alongside.

Serialization and parsing of both file types live in cn-ingest (pure
logic; file I/O in the CLI). Versioned per I7: unknown MAJOR rejected
loudly.

**Cross-implementation test vectors (mandatory before acceptance).**
A test fixture committed to the repo contains:

- A known keypair (TEST ONLY - never used operationally).
- Ciphertexts produced by libsodium.js `crypto_box_seal` for known
  plaintexts, using the test public key.
- Ciphertexts produced by the Rust implementation for the same
  plaintexts.
- The test asserts: JS-sealed opens in Rust; Rust-sealed opens in Rust;
  malformed ciphertext (truncated, wrong key, bit-flipped) fails loudly.

The fixture-generation script (a small Node script using
`libsodium-wrappers`) lives under `scripts/` and is run manually to
refresh vectors; it is NOT part of check-all (no Node dependency in CI).
The Rust test consuming the vectors IS part of check-all.

## 2. Envelope formats (cn-ingest, pure types)

ADR-005 D3 specifies two layers; both are new persisted formats under I7.

**Outer envelope** (cleartext to the relay):

```rust
pub struct OuterEnvelope {
    pub intake_envelope_version: Version,  // semver, I7
    pub recipient_key_fingerprint: String, // fingerprint format
    pub ciphertext: String,                // base64-encoded sealed box
    #[serde(flatten)]
    pub extras: Map<String, Value>,        // unknown-minor preserve
}
```

`OuterEnvelope::parse(bytes) -> Result<Self, IngestError>` with
unknown-MAJOR rejection, unknown-MINOR ignore-and-preserve. Size
validated against a configurable cap BEFORE parsing (D6).

**Inner payload** (plaintext after decryption):

```rust
pub struct InnerPayload {
    pub submission_version: Version,
    pub submission_id: String,        // client UUID, semantic dedup key
    pub form_version: String,
    pub consent: ConsentBlock,
    pub captured_at: String,          // client ISO timestamp
    pub kind: Option<String>,         // payload-carried kind (D-069)
    pub fields: Map<String, Value>,   // form field values
    #[serde(flatten)]
    pub extras: Map<String, Value>,
}

pub struct ConsentBlock {
    pub consent_text_digest: String,
    pub consent_affirmed: bool,       // MUST be true (D-030)
    pub consent_affirmed_at: String,
}
```

`InnerPayload::parse(bytes) -> Result<Self, IngestError>` with the same
version discipline. `consent_affirmed == false` is a loud validation
failure at staging (the form prevents it; seeing it means a hand-crafted
payload).

**Conversion.** `InnerPayload` -> `QueueRecord` with
`SubmissionSource::Remote` populating all remote-only fields from the
pull context (receipt_id, ciphertext_hash, key_used, pulled_at,
relay_received_at, envelope metadata). This is a pure function in
cn-ingest; the puller calls it with the contextual values.

## 3. Keygen CLI commands (ceremony tooling)

New `cn intake` subcommands per the ceremony design sections 3-5.
All live under `core/cli/src/intake/` alongside the existing `apply.rs`
and `queue.rs`.

- **`cn intake keygen`** - generate keypair offline. Prompts for
  passphrase (>= 6 words validated), generates X25519 keypair, writes
  `public.json` and `secret.json` (encrypted) to the specified output
  directory, prints fingerprint. Refuses to overwrite existing key files
  (create-only, like everything else in the queue contract).
- **`cn intake fingerprint <public.json>`** - parse and print the
  fingerprint of a public key file.
- **`cn intake selftest --key-dir <dir>`** - seal a known test vector to
  the public key, open it with the secret key (prompts for passphrase),
  verify round-trip. The `--dry` flag checks that the binary can find
  the crypto implementation without needing key files.
- **`cn intake backup verify [--from-print] <path>`** - decrypt a key
  file from the specified path (USB or printed base32 input), open the
  test vector. `--from-print` accepts base32 text input with CRC check.

All commands produce I12 reports (JSON to stdout, human-readable to
stderr). No network I/O in any of them.

## 4. Pages form (static build artifact)

A new top-level directory `form/` (sibling to `app/` and `core/`). The
form is a SEPARATE build from the app - it deploys to Pages, not the
app's Vite output. It shares no runtime code with the app (the app is
the facilitator tool; the form is the attendee-facing submission
surface).

### 4.1 Form content

A single self-contained HTML page with inline CSS and JS:
- Form fields driven by the group template baked in at build time (kind
  picker if multiple kinds, typed field widgets per R2 attribute types:
  text, number, enum, tags, date, geo as lat/lon, link).
- Consent panel: the D-023-reviewed consent text (DRAFT/PLACEHOLDER
  until sign-off), checkbox gate (unchecked = nothing sends, D-030).
- Submission flow: on submit, the JS constructs the `InnerPayload` JSON,
  vendors `crypto_box_seal` from the embedded libsodium, seals to the
  embedded public key, wraps in `OuterEnvelope` JSON, POSTs to the relay
  origin (the one CSP-allowed connect destination).
- Confirmation screen: "your sealed envelope has been received by the
  relay" - NOT "the facilitator has it" (D6 TTL semantics, D-023
  consent-text implication 4).
- Retry on network error (the relay POST is idempotent from the
  submitter's perspective - a fresh receipt_id each time; the form
  generates a new submission_id per session, so retries after a
  confirmed receipt are new submissions, not replays).
- Fingerprint footer: the embedded key's fingerprint in the `3f9a-...`
  format, visible for out-of-band verification (ceremony design
  section 7).
- Accessibility: labeled fields, keyboard navigation, 375px responsive
  layout (R9). No analytics, no third-party requests.
- CSP: `default-src 'self'; connect-src <relay-origin>; script-src
  'self'; style-src 'self'` (inline CSS/JS are same-origin in the
  bundled artifact).
- No service worker registration (D8 explicit prohibition).

### 4.2 Build pipeline

`scripts/build-form.ps1` (user-facing) / `scripts/build-form.sh`
(internal):

1. Read the group template from `schemas/` and the public key from a
   specified path (or the repo-committed public-key constant).
2. Read the consent text (from `docs/design/` or a specified source).
3. Bundle `libsodium-wrappers` from `node_modules/` (pinned exact
   version in a `form/package.json` with `package-lock.json`; npm
   install is a build prerequisite, not a runtime dependency).
4. Produce a single `index.html` plus any split assets (if the bundler
   requires it; prefer a single file where possible) in `form/dist/`.
5. Emit the **canonical deploy manifest** per ADR-005 D8:
   - JSON, sorted keys, LF newlines, UTF-8 no BOM.
   - Every deployable file: NFC-normalized forward-slash relative path,
     byte length, SHA-256 hash.
   - Sorted by byte-wise path comparison.
   - The manifest itself is NOT in the file set (D8 no-deploy rule).
   - Manifest hash: SHA-256 over the manifest file's exact bytes.
   - Provenance fields: source commit SHA, build timestamp, builder,
     form_version, embedded key fingerprint, consent_text_digest,
     libsodium version + package integrity hash.

Reproducibility: same source commit + same npm lockfile -> same output
bytes. The build pins the libsodium version; no floating dependencies.

### 4.3 Form tooling choice

The form build can be:
- **Minimal: a template-expansion script** that reads template + key +
  consent text and produces HTML with inline JS. No bundler beyond
  string interpolation + libsodium concatenation. Simplest; limited to
  single-file output.
- **Vite (separate config):** a second Vite project under `form/` with
  its own `vite.config.ts`. Heavier but gives tree-shaking, minification,
  and asset hashing for cache busting.

The blueprint leans toward the minimal approach for the pilot: the form
is one page with one dependency (libsodium). A bundler adds build
complexity for marginal benefit at this scale. Implementation may choose
Vite if the single-file approach proves unwieldy.

## 5. Worker relay (Cloudflare Workers + KV)

A new top-level directory `relay/` containing the Worker project
(wrangler.toml, TypeScript source, tests). The relay is operationally
separate from the app and the form; it deploys via `wrangler deploy`.

### 5.1 API surface (ADR-005 D6, normative)

**`POST /submit`** (public, no auth):
- Validates: body size <= configured cap; `Content-Type: application/json`;
  parses outer envelope; `recipient_key_fingerprint` on the admission
  allowlist (D3/D6 rotation cutoff - rejection returns a typed
  `form_out_of_date` error with HTTP 409, telling the form to prompt
  reload).
- Rate limiting: per-source-IP, sized generously for NAT (D6 - hundreds
  of phones behind one venue IP). Throttled -> HTTP 429 with Retry-After.
- Approximate total-blob cap: KV list count (eventually consistent, D6
  honesty); over cap -> HTTP 503 with retry. Not a hard cap.
- On accept: generate 128-bit random receipt_id (server-side, D6);
  write blob to KV (blob TTL), then ledger entry to KV (ledger TTL =
  blob TTL + consistency margin + max pull interval, D6); acknowledge
  ONLY after both writes succeed. Response: `{ "receipt_id": "..." }`.

**`GET /receipts`** (authenticated control plane):
- Bearer token validated against stored hash (constant-time compare, D6
  verifier model). 401 indistinguishable from 404 for unauthenticated.
- Returns paginated list of receipt_ids with metadata (size, arrived_at,
  claimed_fingerprint). KV list with cursor.

**`GET /blob/:id`** (authenticated):
- Returns the raw outer envelope JSON. Repeatable (no destructive read).
- Missing id -> 404. (Indistinguishable from unauthed for public callers.)

**`DELETE /blob/:id`** (authenticated):
- Deletes blob + ledger entry. Idempotent: deleting a missing id
  succeeds with HTTP 204 (distinct from 200 on actual delete).

### 5.2 Worker internals

- **KV namespaces:** two bindings - `INTAKE_BLOBS` (blob prefix + ledger
  prefix) or split into two namespaces if KV pricing favors it. Key
  scheme: `blob:<receipt_id>` for ciphertext, `ledger:<receipt_id>` for
  metadata.
- **Admission allowlist:** a Worker secret or KV config key listing
  accepted fingerprints. Checked on every POST; the rotation cutoff
  (D3/D6) is an allowlist edit.
- **Credential:** a Worker secret storing the SHA-256 hash of the bearer
  token. The bearer token itself is generated at ceremony time and stored
  only in the pilot PC's off-repo config.
- **Logging discipline (D6):** no bodies, no ciphertext, no auth headers
  in logs. Metadata logging minimized; retention set to platform minimum
  in wrangler.toml.
- **No CORS for the control plane.** The form's CSP allows POST to the
  relay origin; the control plane (GET/DELETE) is called from the native
  puller, not a browser.
- **CORS for POST /submit:** `Access-Control-Allow-Origin` set to the
  Pages origin only.

### 5.3 Configuration (deploy-runbook values)

Concrete values are deploy-runbook configuration inside ADR-005's fixed
bounds (D6). The blueprint fixes the config SHAPE; values are set at
deploy:

```toml
# wrangler.toml (indicative)
[vars]
BLOB_TTL_SECONDS = "..."           # >= 2x max pull interval, <= pilot window
LEDGER_TTL_SECONDS = "..."         # blob TTL + consistency margin + pull interval
MAX_BLOB_SIZE_BYTES = "..."        # single-digit KB per D6
APPROXIMATE_BLOB_CAP = "..."       # generous for pilot scale
RATE_LIMIT_PER_IP_PER_MINUTE = "..." # venue-scale NAT safe
PAGES_ORIGIN = "..."               # for CORS
```

Secrets (set via `wrangler secret put`, never in config files):
- `CREDENTIAL_HASH` - SHA-256 of the bearer token
- `ADMISSION_ALLOWLIST` - comma-separated fingerprints

### 5.4 Worker tests

- Unit tests (Vitest + miniflare or Workers test harness):
  POST with valid/invalid envelope, over-size, missing fingerprint,
  wrong fingerprint, rate limit, over cap; authenticated
  list/get/delete; unauthenticated control-plane rejection;
  idempotent delete; ledger write-order (blob first, then ledger);
  CORS headers.
- No integration test against real KV in this blueprint (deploy-barred);
  local miniflare exercises the full API.

## 6. Native puller (`cn intake pull`)

New CLI subcommand under `core/cli/src/intake/pull.rs`. This is the ONE
component that crosses the module fence (D1): it makes HTTPS requests.
The HTTP client dependency (`reqwest` or `ureq`) is added to the CLI
crate only, never to any `cn-*` core crate.

### 6.1 Puller flow (one run)

Preconditions (checked at startup, each recorded in the run report):
1. Queue-root validation (existing: not inside a worktree or cloud-sync
   path, ACL'd, BitLocker active, indexing excluded).
2. Queue lock acquired (existing single-instance lock from
   `cn intake apply`; same lock, same serialization).
3. Key pin check: load the pinned fingerprint from off-repo config;
   compare against the local secret key's public half.
4. Bundle verification (D8): fetch every path in the pinned manifest
   from the deployed origin, cache-bypassed, no redirects, 200 only;
   hash + length match. Extract embedded key fingerprint; match against
   key pin. ANY mismatch -> halt, decrypt nothing, stage nothing, delete
   nothing, non-zero exit. Unreachable origin -> WARN, proceed on
   key-pin-only (D8 degraded path); NO new QR solicitation until bundle
   verifies.

Main loop (per receipt):
1. `GET /receipts` (paginated); iterate.
2. For each receipt_id: transport dedup check - if
   `(receipt_id, ciphertext_hash)` matches an existing queue record,
   recorded no-op (I12), skip to next.
3. `GET /blob/{id}` - fetch outer envelope bytes.
4. Compute `ciphertext_hash` (SHA-256 over raw ciphertext bytes, before
   base64 decode).
5. Parse `OuterEnvelope` (version check, size already validated by relay).
6. Check `recipient_key_fingerprint`: use it to choose which held key to
   try; verify by successful decryption. Fingerprint naming no held key
   -> loud typed error (I3); receipt + ciphertext RETAINED on relay
   (never deleted), flag in run report.
7. `crypto::open(ciphertext, keypair)` - decrypt.
8. Parse `InnerPayload` (version check).
9. Validate consent: `consent_affirmed` must be true; `consent_text_digest`
   checked against known deployed consent-text digests (warning if
   unrecognized, not a halt - the form may have been redeployed between
   submission and pull).
10. Semantic dedup: `(submission_id, payload_hash)` against existing
    queue records. Conflict -> both staged, linked, loud (ADR-005 D4).
11. Stage: construct `QueueRecord` with `SubmissionSource::Remote`,
    write payload record via atomic-write primitive, write initial
    `pending` sidecar, read-back checksum verify.
12. ONLY after verified staging: `DELETE /blob/{id}`. Lost ack -> retry
    next run (idempotent delete).

Post-loop:
- Reconciliation via receipt ledger (D6): `GET /receipts` for ledger
  entries; classify each by the D6 precedence (staged > deleted-by-me >
  blob-present=unpulled > blob-absent-before-TTL=alert >
  blob-absent-after-TTL=expired). Report: per-receipt classification,
  oldest-pending age (half-TTL warning), expired count.
- Run report (I12, JSON): receipts processed, staged, deduped, errors,
  reconciliation summary, bundle-check result, key-pin result,
  precondition status.

### 6.2 HTTP client

`ureq` (blocking, pure Rust, minimal) is the leaning; `reqwest` is the
fallback if TLS or async needs arise. Added to `core/cli/Cargo.toml`
only. The puller constructs requests directly; no HTTP abstraction
beyond what the client crate provides.

TLS: the puller connects to two origins (Pages for bundle check, Worker
for the control plane). Platform TLS (`native-tls` / `rustls`) is an
implementation choice; `rustls` (pure Rust, vendored roots) avoids the
OpenSSL build dependency on Windows.

### 6.3 Puller config

Off-repo JSON config file (versioned per I7, written at ceremony):

```json
{
  "config_version": "0.1.0",
  "key_dir": "<path to public.json + secret.json>",
  "key_fingerprint_pin": "3f9a-1c02-...",
  "relay_origin": "https://<worker>.workers.dev",
  "credential_path": "<path to file holding bearer token>",
  "pages_origin": "https://<org>.github.io/community-connector",
  "manifest_pin": {
    "manifest_hash": "<SHA-256>",
    "commit_sha": "<git SHA>",
    "pinned_at": "<ISO timestamp>",
    "pinned_by": "<operator>"
  },
  "known_consent_digests": ["<SHA-256>", "..."],
  "blob_ttl_seconds": 86400,
  "max_pull_interval_seconds": 3600
}
```

The config path is passed to `cn intake pull --config <path>`. The
config file itself lives in the off-repo facilitator ops directory
(same area as the queue root and key material).

## 7. Permission boundary and PII containment

**Module fence verification.** The implementation must not add any
network dependency to any `cn-*` crate. The review checklist:
`cargo tree -p cn-ingest` (and every core crate) must show no `hyper`,
`reqwest`, `ureq`, `h2`, `http`, or TLS crate. Only `cn` (the CLI
binary) may depend on an HTTP client.

**PII scan extensions.** The existing pii-scan tripwires (D-075) cover
queue records and key envelopes. The relay blueprint adds:
- `relay_credential` or bearer-token-shaped content in any tracked file.
- The form build output (`form/dist/`) is gitignored; the build script
  emits a reminder that dist content must never be committed (the public
  key constant IS committed; the built artifact is NOT - it deploys
  directly to Pages).

**Relay PII posture.** The Worker stores only ciphertext (the outer
envelope). Logged metadata is minimized (D6). The Worker source code
is committed to the repo (world-readable); the secrets (credential
hash, admission allowlist) are set via `wrangler secret` and never
appear in the repo.

## 8. Sequencing (small verified commits, check-all green at each)

### Phase A: Crypto foundation

1. **Sealed-box crypto binding.** Add the chosen crate to cn-ingest;
   implement the `crypto` module (keygen, fingerprint, seal, open, key
   file parsing); cross-implementation test vectors (fixture committed,
   Rust tests in check-all). The JS fixture-generation script under
   `scripts/`.

2. **Keygen CLI commands.** `cn intake keygen`, `cn intake fingerprint`,
   `cn intake selftest`, `cn intake backup verify` (with `--from-print`
   and `--dry`). Tests: round-trip keygen -> selftest; fingerprint
   stability; passphrase-encrypted key round-trip; malformed/wrong-
   passphrase rejection.

### Phase B: Envelope formats and puller logic

3. **Outer/inner envelope types.** `OuterEnvelope`, `InnerPayload`,
   `ConsentBlock` in cn-ingest with version discipline, unknown-minor
   preservation, size-cap pre-check. `InnerPayload` -> `QueueRecord`
   conversion (pure). Tests: version round-trip, unknown-minor
   preserved, oversized rejection, consent_affirmed=false rejection.

4. **Puller core logic (pure, in cn-ingest).** Transport dedup key
   extraction from remote context, receipt classification for
   reconciliation (the D6 precedence table), consent-digest checking.
   Tests: each classification arm, each dedup verdict for remote
   records.

### Phase C: Relay infrastructure

5. **Worker relay.** `relay/` directory: wrangler.toml, TypeScript
   source, Vitest tests with miniflare. Full API surface (POST /submit,
   GET /receipts, GET /blob/:id, DELETE /blob/:id), admission allowlist,
   credential verification, rate limiting, blob+ledger write order,
   CORS. Tests exercise every endpoint and error path.

6. **Pages form build pipeline.** `form/` directory: package.json
   (libsodium-wrappers pinned), build script, template expansion.
   Canonical deploy manifest generation (D8 grammar). `.gitignore` for
   `form/dist/`. Form output tested: fields match template, embedded
   key matches input, CSP present, no external requests, consent
   checkbox gate functional.

### Phase D: Puller I/O and integration

7. **`cn intake pull` CLI command.** HTTP client in the CLI crate only
   (module fence). Precondition checks, main loop (fetch -> decrypt ->
   validate -> stage -> delete), bundle verification (D8 full-bundle
   pin), reconciliation report. Integration test: mock HTTP server
   serving canned outer envelopes -> puller stages QueueRecords with
   SubmissionSource::Remote -> `cn intake apply` approves them ->
   reload verifies graph content. End-to-end on synthetic data.

8. **Bundle verification module.** The D8 manifest fetch-and-compare
   logic as a function in the CLI (not cn-ingest - it needs HTTP).
   Tests: matching manifest passes; missing file, hash mismatch,
   redirect, non-200 each halt; unreachable origin warns.

### Phase E: Cross-cutting

9. **Form-to-graph end-to-end test.** A test script (or a documented
   manual procedure) that exercises: build form -> open in browser ->
   fill fields -> seal and POST to a local mock relay -> puller fetches
   -> decrypts -> stages -> facilitator approves in wizard ->
   `cn intake apply` -> reload -> entity in graph. This is the
   production interactive path for the REMOTE half, complementing the
   in-app rehearsal owed from D-080.

10. **Deploy runbook draft.** `docs/runbooks/intake-relay-deploy.md`:
    ceremony prerequisites, form build + manifest pin, Pages deploy,
    Worker deploy (wrangler), post-deploy verification checklist,
    TTL/cap/rate-limit configuration values with the ADR-005 bounds,
    billing ceiling setup. NOT executed until the deploy bar clears.

11. **Docs true-up.** HANDOFF.md, MANIFEST.md, DEPENDENCIES.md updates
    for new crates, directories, and dependencies. DECISIONS.md entries
    for any deviations during implementation.

Then: the mandatory adversarial round on the implementation diff,
judgment, and acceptance.

## 9. Dependency impact

New workspace dependencies (CLI crate only unless noted):
- Sealed-box crate (cn-ingest): one of `crypto_box`/`dryoc`/`libsodium-sys`
  + `blake2` for fingerprint + `zeroize` for secret key hygiene.
- `argon2` (cn-ingest or CLI): passphrase-based key encryption for the
  ceremony key files.
- HTTP client (CLI only): `ureq` or `reqwest` + a TLS backend.
- `base64` (cn-ingest): sealed-box ciphertext encoding in the outer
  envelope.
- `base32` (CLI only): printed backup format for the ceremony.

New npm dependencies (form only, not the app):
- `libsodium-wrappers` (pinned exact version) - the ONLY runtime
  dependency of the form.

New Node dev dependency (scripts only):
- `libsodium-wrappers` for the cross-implementation test vector
  generator.

Worker dependencies (relay only):
- Cloudflare Workers runtime (no npm packages beyond `@cloudflare/workers-types`
  for TypeScript types). The Worker uses Web Crypto API for credential
  hashing (SHA-256) and random receipt-id generation - no external
  crypto library.

All new dependencies enter DEPENDENCIES.md with version, license, and
purpose before the adversarial round.

## 10. Out of scope

- **Offline/hotspot intake** (D-053 stretch goal, deferred not rejected).
- **Submitter authentication** (D-053: no auth in v0.1.0 or v1.0).
- **Owner-binding** (D-056.2: deferred, triggers authority-matrix change).
- **Per-field tier UX** (post-pilot).
- **Actual deploy** (D-059.8 deploy bar).
- **The keygen ceremony execution itself** (operational, post-D-023).
- **Production interactive load path for the in-app half** (D-080 debt,
  tracked separately; this blueprint addresses the remote half's
  end-to-end rehearsal in step 9).
