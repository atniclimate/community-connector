# Facilitator Keygen Ceremony - Design (D-056.3 / D-056.4)

Status: design draft, 2026-07-24. Director-authored; feeds ADR-005 ("Remote
intake: sealed-envelope relay and facilitator pending-review queue") and its
adversarial round. This is a developer-facing design doc, not community-facing
text (D-051); nothing in it goes to attendees or the committee without D-023
review.

Drivers: D-053 (sealed-envelope relay architecture), D-056.1 (ADR-005 required
scope: key pinning and rotation under I7), D-056.4 (risk register: the
facilitator private key is a single point of total loss; key backup is not repo
data, so no G-BACKUP collision), I1 (no PII, no secrets in the repo), I3 (no
silent failure), I7 (versioned formats, loud rejection).

## 1. Problem

The D-053 remote intake path encrypts every submission in the attendee's
browser to the facilitator's public key (libsodium sealed box). The relay and
the static form never hold anything readable. That design concentrates all
risk in one artifact: the facilitator private key, whose ACTIVE copy is used
only on the pilot PC (two controlled offline recovery copies also exist -
section 5; ADR-005 D3 carries the exact custody and exposure model).

Two failure modes, from D-056.4:

- **Loss.** If the private key is lost (disk failure, theft of the pilot PC,
  accidental deletion), every ciphertext not yet pulled and decrypted is
  permanently unreadable. There is no recovery path by design.
- **Substitution.** If the deployed form is ever changed to embed a different
  public key (compromised deploy, DNS/Pages tampering, an honest redeploy
  mistake), submissions silently encrypt to the wrong key. The facilitator
  must detect this loudly, not discover it by failing to decrypt a batch.

This document specifies the key generation ceremony, the offline backup and
restore procedure, fingerprint pinning in the puller, public-key distribution
and verification, and the rotation/compromise procedure.

## 2. Key material

- Algorithm: libsodium `crypto_box` keypair (X25519), used exclusively for
  `crypto_box_seal` / `crypto_box_seal_open` (anonymous sealed boxes). No
  signing key in v0.1.0; authenticity of submissions is not a goal (no auth,
  D-053) - the facilitator review queue is the abuse gate.
- One keypair per pilot arc. Internal August pilots and the convention MAY use
  distinct keypairs (rotation between them is cheap, section 7); the default
  is one keypair carried through unless a rotation trigger fires.
- Key sizes: 32-byte public key, 32-byte secret key.
- **Fingerprint** (the human-verifiable and machine-pinned identity of the
  key): BLAKE2b-256 over the raw 32-byte public key, truncated to the first
  16 bytes, rendered as 8 lowercase hex groups of 4, e.g.
  `3f9a-1c02-77de-b410-8e55-a0c3-4d21-96fb`. 128 bits is collision-safe for
  pinning and still short enough for a human read-aloud comparison. The
  fingerprint derives from the PUBLIC key only and is safe to commit, print,
  and speak aloud.
- File formats (I7): every key artifact is a small JSON envelope carrying
  `format: "cn-intake-key"`, `schema_version` (semver), `role`
  (`public` | `secret-encrypted`), `created_at`, and `fingerprint`. Readers
  reject unknown MAJOR versions loudly. The raw secret key never exists in an
  unlabeled bare file.

## 3. Tooling: `cn` CLI subcommand (recommended) vs a standalone script

**Recommendation: a `cn intake` subcommand family in the existing Rust CLI.**

Proposed surface (names indicative; final shape lands with ADR-005):

```
cn intake keygen        # generate keypair OFFLINE; writes public + encrypted secret
cn intake fingerprint   # print the fingerprint of a public key file (or the local key)
cn intake backup verify # restore drill: decrypt a self-test vector from a backup medium
cn intake selftest      # seal a test vector to the public key, open it with the secret key
```

Why the CLI and not a small audited script (Node or Python):

1. **One trusted toolchain.** The pilot PC already runs the Rust `cn` binary
   built from a verified commit through the full check-all battery (fmt,
   clippy `-D warnings`, tests). A script adds a second runtime (interpreter
   version, its package manager, its transitive dependencies) to the trusted
   computing base of the single most sensitive artifact in the system - the
   exact supply-chain surface the sealed-envelope design tries to minimize.
2. **The decryption side must be Rust anyway.** The puller opens sealed boxes
   in the core toolchain, so a compatible sealed-box implementation must exist
   in the Rust workspace regardless. Generating the key with the same
   implementation that will open envelopes gives a structural interop
   guarantee and lets `cn intake selftest` exercise the real production path.
3. **Testable and reviewable under the house standard.** A CLI subcommand
   gets unit tests, the AGENTS.md invariant review, and the ADR-005
   adversarial round for free. An "audited script" has no standing harness
   here; its audit would be a one-off.
4. **Offline by construction.** `cn` makes no network calls; a script runtime
   makes that property harder to assert (auto-update checks, telemetry).

Rust binding: the browser side uses libsodium.js `crypto_box_seal`. The Rust
side uses a sealed-box-capable X25519 implementation (candidates: RustCrypto
`crypto_box` with its sealed-box support, or `dryoc`; a libsodium-sys binding
is the fallback if pure-Rust interop fails). The binding choice is an
implementation detail EXCEPT for one hard requirement: **cross-implementation
test vectors** - a fixture of ciphertexts produced by libsodium.js must open
in the Rust implementation, asserted in CI. That test is part of the ADR-005
acceptance, not optional.

## 4. Generation: offline, on the pilot PC

The keypair is generated on the pilot PC with networking disabled (airplane
mode / adapter off) so no window exists in which the secret key coexists with
a live network before backups and verification complete. Steps are in the
section 8 checklist; properties asserted here:

- The secret key is written ONLY in passphrase-encrypted form
  (`secret-encrypted` envelope: XSalsa20-Poly1305 secretbox under an
  Argon2id-derived key, libsodium defaults). There is no plaintext
  secret-key file at rest on the pilot PC; the puller prompts for the
  passphrase and holds the decrypted key in memory only.
- The passphrase is a diceware-style phrase of at least 6 words, generated at
  the ceremony, memorized by the facilitator, and written on the printed
  backup sheet (section 5) - never typed into any file that could be
  committed, synced, or screenshotted.
- The public key file and the fingerprint are non-secret and MAY be committed
  to the repo (the form build needs the public key; section 6).
- All key artifacts on the pilot PC live OUTSIDE the repository working tree,
  in a facilitator ops directory alongside the gitignored staging area the
  pending-review queue uses. Nothing under the repo root ever contains secret
  key material; the pii-scan/secret tripwire treats a `secret-encrypted`
  envelope inside the repo as a blocking finding.

## 5. Offline backup and restore drill (D-056.4)

The private key gets **two offline backups with different failure modes**,
created during the ceremony before the key is ever used:

1. **Printed sheet (plaintext key, physical security).** A single printed
   page containing: the secret key as a QR code AND as base32 text with a
   CRC-style check line (typo-detectable manual re-entry path if the QR is
   damaged); the fingerprint; the ceremony date; the passphrase written by
   hand. Because the sheet IS the key, it goes into a sealed, signed-across-
   the-flap envelope stored in a locked location (locked drawer or safe)
   physically separate from the pilot PC. Rationale for a plaintext copy: a
   backup that itself depends on the passphrase would make the passphrase a
   second single point of total loss; the two backups must not share a
   failure mode.
2. **Encrypted USB.** A fresh USB stick holding the `secret-encrypted`
   envelope (same passphrase-protected format the pilot PC uses) plus the
   public key file. Stored in a second location, separate from both the
   pilot PC and the printed sheet. Losing the USB alone discloses nothing
   without the passphrase.

Storage guidance:

- Three artifacts, three places: pilot PC, printed envelope, USB. No two in
  the same bag, room, or building where practical during the pilot window.
- The printed envelope's location and the USB's location are recorded in the
  facilitator's ops log, which is itself off-repo. **None of this is repo
  data.** G-BACKUP (D-026, the accepted single-machine repo backup risk) is
  about the repository; key backups are operational artifacts handled
  entirely outside git, so accepting G-BACKUP does not accept key loss, and
  solving key backup does not touch the G-BACKUP ruling (D-056.4).
- Backups are destroyed (shredded / securely wiped) when their key is
  retired (section 7).

**Restore drill - mandatory, during the ceremony, before first use:**

1. `cn intake selftest` seals a known test vector to the public key and
   opens it with the live secret key (proves the production decrypt path).
2. `cn intake backup verify` against the USB: read the envelope from the
   USB, prompt for the passphrase, decrypt, open the same test vector.
3. Printed-sheet drill: scan the QR (or type the base32) into
   `cn intake backup verify --from-print`, confirm the check line, open the
   test vector. Then print-artifact hygiene: clear the printer queue and
   confirm the printer has no retained-job storage in use.
4. A backup that fails verification is remade on the spot; the ceremony does
   not complete with an unverified backup. A repeat drill runs once mid-
   window (see checklist) so bit-rot or a lost passphrase is discovered
   before it matters.

## 6. Fingerprint pinning in the puller

The puller (the pilot-PC job that pulls ciphertext batches from the relay,
decrypts, stages into the pending-review queue, then wipes the relay - D-053,
D-056.1) carries a **pinned fingerprint** in its local, off-repo config,
written once at the ceremony.

Before trusting ANY pulled ciphertext batch, the puller (per ADR-005 D8's
measurement procedure, which extends this check from key-only to the whole
bundle):

1. Fetches EVERY file listed in the locally pinned canonical deploy
   manifest (built from the reviewed commit; never fetched from the served
   origin) and verifies each file's bytes-hash and length against the pin,
   then extracts the embedded public key from the verified form.
2. Computes the key's fingerprint and compares against the key pin.
3. Also compares the pin against the fingerprint of its own local secret
   key's public half (catches a stale or mismatched local key after a
   rotation).

**On mismatch: halt loudly.** Typed error (I3), non-zero exit, an unmissable
console banner naming both fingerprints; the puller decrypts nothing, stages
nothing, and - critically - **wipes nothing** on the relay, so no evidence is
destroyed and no submission is lost while the mismatch is investigated. A
mismatch means either an unrecorded redeploy or key substitution; both demand
a human before any further automated action. There is no override flag; the
only fix is correcting the deployed form or re-pinning after a verified,
logged rotation (section 7).

If the deployed origin is unreachable (offline pilot PC, Pages outage), the
puller may proceed on the local-key-vs-pin check alone but must print a
WARN that the deployed-bundle check was skipped; skipping is never silent
(I3, I12), and NO new solicitation (QR presentation) happens until the
deployed bundle verifies against the pin (ADR-005 D8 - the bundle check
protects future submitters; already-sealed ciphertext is not endangered by
current-bundle state).

## 7. Public-key distribution and human verification

Distribution at deploy time:

- The public key and fingerprint are checked into the repo as a versioned
  constant consumed by the static form build (public material only; the
  fingerprint doubles as the human-auditable identity in code review).
- The form build embeds the key and stamps the fingerprint into a visible
  diagnostic on the form itself - a small footer line such as
  "intake key: 3f9a-1c02-..." - and emits the canonical deploy manifest
  (ADR-005 D8 grammar). The manifest is NOT deployed (ADR-005 D8): the
  deployed set is exactly the manifest's listed files; the puller
  verifies only against the locally pinned manifest recorded at deploy.
  Rendering the fingerprint where any submitter could in principle
  compare it costs nothing and makes the out-of-band check trivial.

Out-of-band human verification (after every deploy that touches the form):

1. On a device OTHER than the pilot PC (a phone on cellular is ideal - a
   different machine and network path than the deploy), load the deployed
   Pages form.
2. Read the footer fingerprint aloud against the printed ceremony sheet's
   fingerprint (or `cn intake fingerprint` output on the pilot PC). All
   eight groups must match.
3. Record the check (date, deploy id/commit, verifier) in the off-repo ops
   log. The relay does not go live for attendees (no QR distribution) until
   this check has passed at least once for the current deploy.

## 8. Rotation and compromise procedure

Plain statement first, because it is the whole reason this document exists:
**sealed boxes have no escrow. If the private key is lost, every ciphertext
sitting on the relay or in any un-decrypted pulled batch is permanently
unreadable - those submissions are gone and the people who submitted them
will not appear in the graph unless they submit again.** Already-decrypted,
already-staged, or already-approved entries are unaffected (they live in the
queue/op log, not under the key).

### 8.1 Key LOST mid-pilot (pilot PC dies, both backups fail)

1. Stop distributing the QR / take the form down (swap in a static "intake
   paused" page) so no new submissions encrypt to the dead key.
2. Record the loss window in the ops log; count the ciphertexts stranded on
   the relay (count only - they are noise now).
3. Run the full ceremony again (new keypair, new backups, new drill).
4. Update the repo constant, rebuild and redeploy the form, re-run the
   section 6 out-of-band verification, re-pin the puller.
5. Wipe the stranded ciphertexts from the relay (they are undecryptable to
   everyone, including us; TTL would eventually clear them anyway).
6. Resume QR distribution. If feasible, ask on-site facilitation to invite
   affected submitters to re-submit; identifying WHO was stranded is
   impossible by design, so this is a broadcast ask, and any community-facing
   wording for it is D-023 human-review territory.

### 8.2 Key LEAKED (secret key disclosure suspected or confirmed)

A leaked key lets the holder read ciphertexts they can obtain - anything on
the relay now, anything captured in transit historically, and all future
submissions until rotation. It does NOT let them tamper with the graph
(submissions still land in the pending-review queue; the facilitator gate
stands).

1. Immediately pull and durably stage everything currently on the relay,
   then wipe it (normal pull path - this shrinks the exposed set).
2. Take the form down / pause QR distribution.
3. Run the full ceremony again on a machine believed clean; destroy the old
   key's printed sheet and wipe the old USB.
4. Redeploy the form with the new key, out-of-band verify, re-pin the
   puller.
5. Treat every submission from the suspected exposure window through the
   rotation as potentially disclosed. Escalate to the human: disclosure
   assessment and any notification to the committee or submitters is a
   human/governance call (D-023, D-034 authority), never autonomous.
6. Log the incident (window, suspected vector, actions) in the ops log and
   a DECISIONS.md entry for the rotation ruling itself (no key material, no
   locations, in the repo entry).

### 8.3 Planned rotation (e.g. between August pilots and the convention)

Same steps as 8.1 minus the loss accounting: drain the relay under the old
key FIRST (pull, decrypt, stage, wipe), then ceremony, redeploy, verify,
re-pin. Zero submissions are stranded if the relay is drained before the
form flips. Destruction timing follows ADR-005 D3's cutoff rule - NOT
"as soon as the last visible ciphertext is staged": after the relay
admission allowlist drops the old fingerprint (a stale open tab then gets
a visible "reload the form" rejection instead of silently sealing to a
dead key), the old key and its backups are retained for one further relay
TTL and destroyed only after ledger reconciliation shows no unaccounted
old-key receipt.

## 9. Ceremony checklist

Executed by the facilitator on the pilot PC. Print this section; fill blanks
by hand; file the completed sheet in the off-repo ops log. Every VERIFY line
is a stop-on-fail.

```
Ceremony date: ____________   Operator: ____________
cn build commit: ____________ (must be a check-all-green commit)

PREP
[ ] 1.  Pilot PC OS current; disk encryption (BitLocker or equivalent) ON.
[ ] 2.  cn binary built from the commit above; `cn intake selftest --dry`
        runs clean.
[ ] 3.  Printer available and working; NOT a shared/managed print server.
[ ] 4.  One fresh USB stick, never used elsewhere, on hand.
[ ] 5.  Opaque envelope + pen on hand.
[ ] 6.  ALL networking disabled (Wi-Fi off, Ethernet unplugged, Bluetooth
        off). VERIFY: no adapter shows connected.

GENERATE
[ ] 7.  Run `cn intake keygen`. Choose a fresh 6+ word passphrase; say it
        nowhere, type it only at the prompt.
[ ] 8.  Record the fingerprint here, by hand, all 8 groups:
        ____-____-____-____-____-____-____-____
[ ] 9.  VERIFY: `cn intake selftest` passes (seal + open round trip).

BACKUPS
[ ] 10. Print the backup sheet (QR + base32 + check line + fingerprint).
        Write the passphrase on it BY HAND. VERIFY: printed fingerprint
        matches line 8.
[ ] 11. VERIFY: `cn intake backup verify --from-print` passes from the
        printed sheet (scan or type the base32).
[ ] 12. Clear the printer queue; confirm no retained job storage.
[ ] 13. Seal the sheet in the envelope; sign across the flap; note the
        intended storage location in the ops log (not here, not in repo).
[ ] 14. Write the encrypted envelope + public key to the USB.
[ ] 15. VERIFY: `cn intake backup verify` passes against the USB, on a
        fresh mount, passphrase from memory.
[ ] 16. Confirm the two backup locations are distinct from each other and
        from the pilot PC.

PIN AND PUBLISH
[ ] 17. VERIFY: no key artifact exists under the repository working tree
        except the public-key constant staged for commit; pii-scan passes.
[ ] 18. Write the fingerprint into the puller's local pinned-fingerprint
        config (off-repo). VERIFY: puller startup check passes against the
        local key.
[ ] 19. Re-enable networking.
[ ] 20. Commit the public-key constant. BUILD LOCALLY from that reviewed
        commit (reproducible build); the build emits the canonical deploy
        manifest (ADR-005 D8 grammar). Inspect it (file list sane, key
        constant present, no unexpected entries).
[ ] 21. PIN: record the manifest's exact bytes and its SHA-256, plus
        provenance (commit SHA, operator, time), in the off-repo ops
        config and ops log. The pinned copy is the sole verification
        authority.
[ ] 22. Deploy EXACTLY the built file set (only if D-053/D-055 publish
        preconditions currently hold; otherwise stop here and park - the
        ceremony through line 21 is complete and valid).
[ ] 23. VERIFY: fetch every path listed in the PINNED manifest from the
        deployed origin (cache bypassed, no redirects, status 200,
        identity bytes) and match each hash and length; then the embedded
        key fingerprint against the pin. Any mismatch stops the line.
[ ] 24. VERIFY (out-of-band): on a different device and network, load the
        deployed form; its footer fingerprint matches line 8, all groups.
[ ] 25. VERIFY: full puller pre-run gate (bundle + key + local-key pins)
        passes end to end.

CLOSE
[ ] 26. File this sheet + backup locations + verifier initials in the
        off-repo ops log.
[ ] 27. Schedule the mid-window repeat restore drill (steps 11 and 15)
        for: ____________ (date roughly halfway to the convention).
```

## 10. Open questions (for ADR-005 and its adversarial round)

- Rust sealed-box binding choice (RustCrypto `crypto_box` vs `dryoc` vs
  libsodium-sys) - decided by the cross-implementation test vectors, not by
  preference.
- ~~Whether the deploy manifest should be signed with a second key~~
  RESOLVED by ADR-005 rounds 1-3: the served manifest is non-authoritative
  and never read as an authority, so signing it protects nothing; the
  locally built, off-origin pinned manifest is the trust root (ADR-005 D8,
  rejected option 9).
- Exact placement and format of the puller's pinned-fingerprint config
  within the off-repo facilitator ops directory (belongs with the pending-
  review queue persistence design, D-056.1).
- Whether internal August pilots and the convention use one keypair or two
  (default one; a planned 8.3 rotation between them is cheap if the
  committee prefers it).
