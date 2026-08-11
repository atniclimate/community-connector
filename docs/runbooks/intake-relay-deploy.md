# Runbook: intake-relay deploy (DRAFT - NOT to be executed)

Blueprint `docs/blueprints/intake-relay.md` section 8 step 10, Phase E; ADR-005
D2/D3/D6/D8; keygen ceremony `docs/design/facilitator-keygen-ceremony.md`. This
is the LIVE-deploy counterpart to the synthetic-only local rehearsal in
`docs/runbooks/e2e-remote-intake.md`.

Status: DRAFT. This runbook provisions real infrastructure (a public GitHub
Pages form and a live Cloudflare Worker relay that will accept real attendee
submissions). It MUST NOT be run until the DEPLOY bar clears (next section).
Nothing in this document is a decision to deploy; it is the procedure to follow
AFTER the human opens the last gate conditions.

This file is committed to a PUBLIC repository. It contains NO keys, tokens,
hashes, account ids, namespace ids, or origins. Every such value is shown as a
command that GENERATES or SETS it, never as a literal. Do not paste a real
secret, key, account id, or PII into this file or into any commit.

All user-facing commands are PowerShell (this is a Windows workspace), run from
the repository root unless noted.

## STOP - the DEPLOY bar (D-059.8) must clear first

The deploy bar is a human gate. It has four conditions; do not start Step 1
until all four are recorded as met in HANDOFF.md.

| # | Condition | Status |
|---|---|---|
| 1 | ADR-005 accepted | DONE (D-068, eight adversarial rounds, 2026-07-24) |
| 2 | Intake pipeline working end to end on synthetic data | DONE (relay blueprint Phases A-E landed and rehearsed; `docs/runbooks/e2e-remote-intake.md`) |
| 3 | Keygen ceremony executed (real facilitator keypair generated, backed up, drill passed) | PENDING (`docs/design/facilitator-keygen-ceremony.md` section 9 checklist) |
| 4 | D-023 sign-off on the community-facing form text (consent panel, confirmation wording, tier wording) | PENDING (human review; the form ships DRAFT consent text until then) |

Conditions 3 and 4 are human/committee acts. They are NOT closed by any agent,
by tests passing, or by momentum. Until both are recorded as met, this runbook
stays a draft and no `wrangler deploy` and no Pages publish runs.

Adjacent standing gates that also govern this deploy:

- The remote/publishing, hosting-vendor, and spend gates were opened by the
  human for EXACTLY ONE path: the public repo `atniclimate/community-connector`,
  the GitHub Pages intake form, and the Cloudflare Workers relay (CLAUDE.md gate
  notes, D-053). This runbook provisions that one path and nothing else. Do not
  provision any additional paid service, second Worker, or other vendor under
  cover of this runbook.
- The keygen ceremony (condition 3) is itself gated on D-023 and is operational,
  post-sign-off work (blueprint section 10 "out of scope").

## Configuration values and their ADR-005 D6 bounds

The relay's `[vars]` in `relay/wrangler.toml` are ILLUSTRATIVE PLACEHOLDERS
(the file says so). At deploy the operator chooses real values INSIDE the D6
bounds below. Choose the committed maximum pull interval and the pilot-window
length FIRST; the TTLs derive from them.

| Var | ADR-005 D6 bound | Source of truth | Illustrative value |
|---|---|---|---|
| `BLOB_TTL_SECONDS` | >= 2x the committed max pull interval, and <= the pilot-window length | `relay/src/env.ts` (line 12), `relay/wrangler.toml` (line 32) | `86400` (24h) |
| `LEDGER_TTL_SECONDS` | blob TTL + KV consistency margin + at least one max pull interval (the D6 guaranteed observation window) | `relay/src/env.ts` (line 14), `relay/wrangler.toml` (lines 34-37) | `93600` (26h) |
| `MAX_BLOB_SIZE_BYTES` | single-digit KB | `relay/src/env.ts` (line 16), `relay/wrangler.toml` (line 38) | `8192` (8 KiB) |
| `APPROXIMATE_BLOB_CAP` | generous for pilot scale (~150 expected / 300 max); an eventually-consistent list-count backstop, NOT a hard cap | `relay/src/env.ts` (line 18), `relay/wrangler.toml` (line 43) | `1000` |
| `RATE_LIMIT_PER_IP_PER_MINUTE` | venue-scale NAT-safe (hundreds of legitimate phones behind one venue IP); sized generously per-IP | `relay/src/env.ts` (line 20), `relay/wrangler.toml` (line 46) | `600` |
| `PAGES_ORIGIN` | the EXACT deployed Pages origin, for CORS on `POST /submit`; never `*` | `relay/src/env.ts` (line 22), `relay/wrangler.toml` (line 50) | `<to-confirm real origin>` |

Service-objective inputs to the TTL choice (ADR-005 D6): the puller runs at
least DAILY during the August internal pilots and at least HOURLY during the
convention window. The committed max pull interval for the active window is the
`2x` multiplier's basis - a daily cadence forces `BLOB_TTL_SECONDS >= 172800`,
an hourly cadence only `>= 7200`. The illustrative `86400` assumes a pull
interval of 12h or less; confirm the real interval before pinning the number.

Cross-fence constraint (D-086, BINDING): the puller's `max_envelope_bytes`
(its `PullConfig` field) MUST be `>=` the relay's `MAX_BLOB_SIZE_BYTES`. A
puller cap below the relay cap would silently reject a valid, consented
submission the relay accepted - lost consented data over a config mismatch.
Set them equal (both `8192` by default) or raise the puller side. See Step 8.

Two secrets are NEVER in any file (set via `wrangler secret put`, Step 5):

- `CREDENTIAL_HASH` - the SHA-256 hex of the control-plane bearer token
  (verifier model, D6; `relay/src/env.ts` line 26). The relay stores only the
  hash; the pilot PC holds the token.
- `ADMISSION_ALLOWLIST` - comma-separated admitted key fingerprints; for a
  single-key pilot this is exactly the ceremony key's fingerprint
  (`relay/src/env.ts` line 28; the D3/D6 rotation cutoff is an edit to this
  list).

## Prerequisites

- Deploy bar (above) recorded as fully met in HANDOFF.md.
- Keygen ceremony COMPLETE (ceremony checklist through line 21). In hand from
  it: the facilitator `public.json` (public key hex + fingerprint) and the
  fingerprint recorded off-repo. The SECRET key never leaves the pilot PC / the
  offline backups and never enters this procedure.
- A GitHub account with push/Pages rights on `atniclimate/community-connector`,
  under the org's custody rules already in force for the D-060 push credential.
- A Cloudflare account under the organization's control, with 2FA enabled and
  recovery factors recorded in the off-repo ops log (ADR-005 D6 "Vendor
  operational acceptance"). Cloudflare service terms reviewed at deploy.
- Toolchain: Node 24 (`node --version`); `npm install` completed in `form/`; the
  Rust `cn` CLI built from a check-all-green commit
  (`cargo build --manifest-path core/Cargo.toml -p cn`); `wrangler`
  (`^4.120.1`, from `relay/package.json`) available via `relay/node_modules`.
- Both origins decided up front (they are cross-referenced): the Pages origin
  `https://<org>.github.io/community-connector` and the relay origin
  (`https://<worker-name>.<account-subdomain>.workers.dev` or a custom domain).
  The form build bakes in the relay origin; the relay build takes the Pages
  origin for CORS. Neither literal is committed to this public repo.

## Step 1 - Build the form and pin the D8 manifest

The form is a build artifact, not hand-written HTML. Build it reproducibly from
the reviewed, check-all-green commit, embedding the REAL ceremony public key
(never the secret) and the real relay origin. The build-time inputs are the
`CN_FORM_*` env vars read by `form/vite.config.ts`.

```powershell
# From the ceremony public.json (off-repo). Public material only.
$pubHex      = "<facilitator public key hex from public.json>"
$fingerprint = "<facilitator key fingerprint from public.json>"
$relayOrigin = "https://<worker-name>.<account-subdomain>.workers.dev"
$formVersion = "<pilot form version, e.g. atni-convention-2026-09>"

$env:CN_FORM_PUBLIC_KEY_HEX  = $pubHex
$env:CN_FORM_KEY_FINGERPRINT = $fingerprint
$env:CN_FORM_RELAY_ORIGIN    = $relayOrigin
$env:CN_FORM_VERSION         = $formVersion
# CN_FORM_TEMPLATE_PATH: set only if the pilot template differs from the
# committed default (form/vite.config.ts DEFAULT_TEMPLATE_PATH).

Push-Location form; npm run build; Pop-Location   # -> form/dist (gitignored)
```

`form/dist` is gitignored and MUST NEVER be committed - it deploys directly to
Pages. Now emit and PIN the canonical deploy manifest (D8). The manifest is
written OUTSIDE the deploy set and is NOT itself deployed. Write it to the
off-repo ops directory, not into the repo.

```powershell
$ops      = "<off-repo facilitator ops directory>"
$manifest = Join-Path $ops "dist.manifest.json"
$commit   = (git rev-parse HEAD)

node form/scripts/gen-manifest.mjs --dist form/dist --out $manifest `
  --fingerprint $fingerprint --form-version $formVersion --commit $commit
# Prints manifest_sha256, consent_text_digest, key_fingerprint.

$manifestHash = (Get-FileHash -Algorithm SHA256 $manifest).Hash.ToLower()
```

Pin, per ADR-005 D8 and ceremony checklist line 21: record the manifest's exact
bytes and its SHA-256, plus provenance (commit SHA, deployer, timestamp), in the
off-repo ops config AND in a DECISIONS.md deploy entry (the public,
version-controlled anchor). The LOCALLY built manifest is the sole verification
authority; NEVER re-derive the pin from the served origin (that would recreate
the same-origin trust hole D8 exists to close). Inspect the file list: exactly
the expected form files, the key constant present, no unexpected entries.

## Step 2 - Deploy the form to GitHub Pages (NOT Cloudflare Pages)

Per ADR-005 D2 the form is served from GitHub Pages out of the public repo.
Cloudflare hosts ONLY the relay (Step 7). Do not deploy the form to Cloudflare
Pages.

Deploy EXACTLY the built file set from Step 1 - nothing more (D8: the deployed
set equals the manifest's listed files). Because `form/dist` is gitignored and
must not be committed, publish it as a GitHub Pages BUILD ARTIFACT from the
reviewed commit rather than by committing built files:

- Pages source = "GitHub Actions": a workflow builds the form from the reviewed
  commit with the same `CN_FORM_*` inputs and uploads `form/dist` as the Pages
  artifact, so no built file is ever committed. Branch protection on the
  deploying branch; deploy only from reviewed commits (D8 "Repo/deploy-chain
  protections"). The exact workflow file does not exist yet - `<to-confirm>`:
  author it, or publish the artifact by hand via the Pages deployment API, but
  in EITHER case the served bytes must reproduce the locally pinned manifest.

Because the pin is the LOCAL build (Step 1), verify the deployed origin against
that pin AFTER publish (Step 9), never the reverse. Two deploy considerations to
confirm before publish:

- `<to-confirm>` Base path: a GitHub Pages PROJECT site serves under the
  `/community-connector/` subpath, but `form/vite.config.ts` sets no Vite
  `base`, so asset URLs default to `/`. Confirm the serving path (custom domain,
  user/org root site, or a `base` set at build) so `assets/*` resolve; a
  subpath mismatch 404s the bundle. Resolve this before Step 1's build if a
  `base` is required, since it changes the built bytes and therefore the pin.
- No service worker is registered and none is ever introduced on this origin
  (ADR-005 D8 explicit prohibition).

## Step 3 - Create the KV namespace

One KV namespace, two key prefixes (`blob:<id>`, `ledger:<id>`), per
`relay/wrangler.toml`.

```powershell
Push-Location relay
node node_modules/wrangler/bin/wrangler.js kv namespace create INTAKE_BLOBS
Pop-Location
# Prints the namespace id.
```

Put the returned id into the `INTAKE_BLOBS` binding in your LOCAL working copy
of `relay/wrangler.toml`, and set `account_id`. The committed file keeps its
`REPLACE_WITH_*` placeholders; do NOT commit real account/namespace ids to this
public repo (keep the edit in the working tree only, or provide `account_id`
via `$env:CLOUDFLARE_ACCOUNT_ID`). Namespace ids and account ids are
account-scoped identifiers, not secrets, but the committed file deliberately
holds no real account - keep it that way unless the maintainer decides
otherwise.

## Step 4 - Set the relay vars (within the D6 bounds)

Choose each `[vars]` value from the bounds table above. Either edit them in your
LOCAL working copy of `relay/wrangler.toml`, or pass them inline at deploy with
`--var NAME:value` (Step 7). `PAGES_ORIGIN` is the exact origin from Step 2 and
is site-identifying - keep it out of committed history.

```powershell
# Example inline values chosen within the D6 bounds (confirm each against the
# committed max pull interval and pilot window before real deploy):
#   BLOB_TTL_SECONDS            <to-confirm: >= 2x max pull interval, <= window>
#   LEDGER_TTL_SECONDS          <to-confirm: blob TTL + margin + >= 1 interval>
#   MAX_BLOB_SIZE_BYTES         8192   (single-digit KB; matches puller default)
#   APPROXIMATE_BLOB_CAP        1000   (generous for ~150-300 pilot scale)
#   RATE_LIMIT_PER_IP_PER_MINUTE 600   (venue-scale NAT-safe)
#   PAGES_ORIGIN                https://<org>.github.io/community-connector
```

## Step 5 - Set the relay secrets (never in any file)

Both secrets are set via `wrangler secret put` and live only in Cloudflare's
secret store and the pilot PC's off-repo ops config. Never in the repo, the form
bundle, or any log.

```powershell
Push-Location relay

# CREDENTIAL_HASH = SHA-256 hex of a freshly generated control-plane bearer
# token. Generate the token, store the TOKEN off-repo on the pilot PC, and give
# the relay only its HASH.
$token = -join ((1..48) | ForEach-Object { '{0:x}' -f (Get-Random -Max 16) })
$token | Set-Content -NoNewline "<off-repo path>\relay-credential.txt"
$credHash = (Get-FileHash -Algorithm SHA256 -InputStream `
  ([IO.MemoryStream]::new([Text.Encoding]::UTF8.GetBytes($token)))).Hash.ToLower()
$credHash | node node_modules/wrangler/bin/wrangler.js secret put CREDENTIAL_HASH

# ADMISSION_ALLOWLIST = the ceremony key fingerprint (comma-separated if more
# than one key is admitted; a single-key pilot has exactly one).
$fingerprint | node node_modules/wrangler/bin/wrangler.js secret put ADMISSION_ALLOWLIST

Pop-Location
```

The token file is off-repo operational material (same area as the queue root
and key material). It is the puller's `credential_path` in Step 8.

## Step 6 - Configure the billing ceiling and alerts (spend gate)

The spend gate was opened by the human for EXACTLY this relay (CLAUDE.md gate
notes, D-053). ADR-005 D6 makes the billing ceiling a DEPLOY GATE: the true hard
bounds on storage and spend are the per-blob size cap, the TTL (storage
self-drains), platform quotas, and the configured billing ceiling plus alerts -
the approximate blob cap is only a soft backstop.

- Pilot scale (~150 expected / 300 max submissions of single-digit KB) is stated
  in D6 as trivially inside free-tier KV. Stay on the free tier where the pilot
  fits; do not enable paid add-ons beyond what the relay needs.
- Configure Cloudflare billing NOTIFICATIONS/alerts at a low threshold and
  record the numeric ceiling and alert thresholds in the off-repo ops log AND a
  DECISIONS.md deploy entry (D6 requires the numeric values recorded as a deploy
  gate). `<to-confirm>` the exact figures - the maintainer sets them under the
  opened spend gate; they are not an agent decision.
- Honesty note (D6): Cloudflare usage-based billing may have no hard auto-cutoff
  at the dollar level; alerts are the monitoring layer, and the size cap + TTL +
  facilitator review gate are the real abuse backstops. Overload degrades to
  "remote intake temporarily refuses; in-app entry continues."
- Record the vendor operational preconditions (account under org control, 2FA,
  recovery factors, terms reviewed) in the ops log per D6.

## Step 7 - Deploy the relay to Cloudflare Workers

Dry-run first (this is the committed `build-check` script), then deploy.

```powershell
Push-Location relay
npm run build-check                                   # wrangler deploy --dry-run
# Real deploy (vars either in the working-tree wrangler.toml or inline --var):
node node_modules/wrangler/bin/wrangler.js deploy
Pop-Location
```

The Worker source is world-readable in the repo; only the secrets (Step 5) and
the account/namespace ids (Step 3) are not. Confirm after deploy that the
Worker's `[vars]` resolved to the intended values (a nonsense value fails closed
as a generic 500 by `relay/src/lib/config.ts`, never silently as 0/NaN).

## Step 8 - Wire the puller config (off-repo)

Write the puller config in the off-repo ops directory (blueprint 6.3 shape,
extended by D-085 `manifest_path` and D-086 `max_envelope_bytes`). It pins the
key fingerprint and the LOCAL manifest hash from Step 1.

```powershell
@{
  config_version        = "0.1.0"
  key_dir               = "<off-repo dir holding public.json + secret.json>"
  key_fingerprint_pin   = $fingerprint
  relay_origin          = $relayOrigin
  credential_path       = "<off-repo path>\relay-credential.txt"
  pages_origin          = "https://<org>.github.io/community-connector"
  manifest_path         = $manifest
  manifest_pin          = @{ manifest_hash = $manifestHash; commit_sha = $commit;
                             pinned_at = (Get-Date).ToString("o"); pinned_by = "<operator>" }
  known_consent_digests = @("<consent_text_digest from Step 1 output>")
  blob_ttl_seconds      = <to-confirm: match relay BLOB_TTL_SECONDS>
  max_pull_interval_seconds = <to-confirm: the committed cadence for this window>
  max_envelope_bytes    = 8192   # D-086: MUST be >= relay MAX_BLOB_SIZE_BYTES
} | ConvertTo-Json -Depth 8 | Set-Content "<off-repo path>\pull-config.json"
```

D-086 check before first live pull: `max_envelope_bytes` (`8192`) >= the relay's
deployed `MAX_BLOB_SIZE_BYTES`. If Step 4 raised the relay cap, raise this to
match.

## Step 9 - Post-deploy verification (before any QR goes live)

Do all of these BEFORE presenting a QR to any attendee. Any failure halts the
line - no solicitation until it verifies (ADR-005 D8, ceremony section 6).

1. Out-of-band fingerprint check (ceremony section 7 / checklist line 24): on a
   DIFFERENT device and network path than the pilot PC, load the deployed Pages
   form and read its footer fingerprint aloud against the printed ceremony
   sheet. All eight groups must match. Record (date, commit/deploy id, verifier)
   in the off-repo ops log.
2. D8 bundle verification (checklist line 23): the puller fetches every path in
   the PINNED manifest from the deployed Pages origin, cache-bypassed, no
   redirects, status 200, identity bytes; each hash and length must match the
   pin, and the embedded key fingerprint must match the key pin. A synthetic
   dry pull against the live relay exercises this:

   ```powershell
   cargo run --manifest-path core/Cargo.toml -p cn -- intake pull `
     --config "<off-repo path>\pull-config.json" --queue "<off-repo scratch queue>"
   # Expect preconditions.bundle_check.status = "verified".
   ```

3. End-to-end synthetic sealed envelope (submit -> pull -> confirm wipe). Use
   FICTIONAL data only; every email ends in `@example.test` (I1):
   - On a phone, open the deployed Pages form, fill SYNTHETIC fields, affirm the
     consent checkbox, submit. Confirm the form shows a receipt id and the
     confirmation says the RELAY received it (not that the facilitator has it).
     A `409 form_out_of_date` here means the built form's fingerprint is not on
     the relay `ADMISSION_ALLOWLIST` - fix Step 5.
   - Run `cn intake pull` (as above) on the pilot PC. Expect the run report:
     `bundle_check.status = "verified"`, `staged = 1`, `blobs_deleted = 1`,
     `errors = []`, and a staged record with `source.kind = "remote"`.
   - Confirm the WIPE: the puller deletes the relay blob only AFTER verified
     durable staging (ADR-005 D4). A second `cn intake pull` must stage nothing
     new (`staged = 0`) and the reconciliation summary must show the receipt as
     deleted-by-me, i.e. the relay no longer holds that ciphertext.
   - Optionally carry it through `cn intake apply` and `export` exactly as the
     local rehearsal does, to confirm the synthetic entry reaches the graph,
     then discard the scratch queue and ops.
4. Purge the synthetic test: delete the scratch queue and any test envelope so
   no test data lingers. Confirm KV holds no leftover synthetic blob.

Only after 1-4 pass does the QR go live for attendees.

## Rollback and teardown

- Pause intake: take the Pages form down (swap in a static "intake paused" page)
  and/or remove the key fingerprint from `ADMISSION_ALLOWLIST` (an out-of-date
  form then gets a visible `409 form_out_of_date`, never silent loss). In-app
  facilitator entry is the always-available fallback (ADR-005 D2).
- Rotate the control-plane credential at each pilot-window boundary and
  immediately on any suspected exposure (`wrangler secret put CREDENTIAL_HASH`
  with a fresh token; update the puller `credential_path`).
- Key rotation/compromise follows ADR-005 D3 and ceremony section 8 (drain,
  ceremony, redeploy, out-of-band verify, re-pin; destroy old key only after the
  completed cutoff epoch + one relay TTL + clean ledger reconciliation).
- End of window: reconcile the relay to zero, verify KV holds zero blobs (D6
  post-sweep check), then run the recorded close-of-window sweep (D4/D-059.11)
  on the off-repo queue.

## Notes

- Form host is GitHub Pages; relay host is Cloudflare Workers. A prior doc
  slipped and called the form host Cloudflare Pages - it is not. Keep them
  distinct.
- The local, synthetic-only counterpart to this runbook is
  `docs/runbooks/e2e-remote-intake.md`. It runs everything on throwaway keys and
  a local relay and never crosses the deploy bar; use it to rehearse the flow
  before touching this one.
- Every config number here traces to `relay/src/env.ts` / `relay/wrangler.toml`
  and ADR-005 D6. Values marked `<to-confirm>` are deploy-runbook choices the
  operator/maintainer sets within the stated bound at deploy time - not guessed
  here.
