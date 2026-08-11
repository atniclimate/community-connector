# Runbook: remote-intake form-to-graph rehearsal (manual, browser-driven)

Blueprint `docs/blueprints/intake-relay.md` section 8 step 9, Phase E; ADR-005
D1/D3/D4/D6/D8. Status: SYNTHETIC-data rehearsal only. This is NOT a deploy and
NOT the DEPLOY bar (D-059.8) - it runs everything locally on throwaway keys.

## What this is (and how it differs from the automated script)

Two rehearsals exercise the remote half end to end. They are complementary:

| | `scripts/e2e-remote-intake.ps1` (automated) | This runbook (manual) |
|---|---|---|
| Browser | none (seals via the form's crypto in Node) | real browser, real form UI |
| Facilitator approval | `emit-approve-decision` example | the P3.5 wizard in the app |
| D8 bundle check | `pages_origin` closed -> **Degraded** (proceeds on key-pin) | `form/dist` served -> **Verified** (full-bundle pin matches) |
| Runs in | CI-style one shot, no interaction | by hand, ~10 minutes |

Run the automated script first for the fast regression signal; run this when you
want to see the production interactive path (real form fill, real wizard approve)
and to exercise the **Verified** D8 path the automated script deliberately skips.

PII posture (I1): every field you type is FICTIONAL and every email ends in
`@example.test`. Nothing you produce here is committed - keys, queue, relay state,
and the served form build all live outside the git worktree.

## Prerequisites

- Node 24 (`node --version`), with deps installed in `form/`, `app/`, and
  `relay/` (`npm install` in each if `node_modules` is absent).
- Rust toolchain; build the CLI once: `cargo build --manifest-path core/Cargo.toml -p cn`.
- A scratch directory OUTSIDE the repo, e.g. `$work = "$env:TEMP\cn-remote-rehearsal"`.

All commands below are PowerShell, run from the repo root unless noted.

## Procedure

### 1. Generate a throwaway ceremony key

```powershell
$work = "$env:TEMP\cn-remote-rehearsal"; New-Item -ItemType Directory -Force $work | Out-Null
$keys = Join-Path $work "keys"
cargo run --manifest-path core/Cargo.toml -p cn -- intake keygen --out $keys
# Enter a >= 6-word passphrase twice. Note the fingerprint it prints.
$pub = Get-Content (Join-Path $keys "public.json") -Raw | ConvertFrom-Json
$fingerprint = $pub.fingerprint
$pubHex = $pub.public_key
```

### 2. Build the form pinned to this key and a local relay origin

The recipient key, its fingerprint, and the relay origin are compile-time
constants injected by `form/vite.config.ts` from env vars (there are no runtime
fetches). Choose a relay port (default below 8799):

```powershell
$env:CN_FORM_PUBLIC_KEY_HEX  = $pubHex
$env:CN_FORM_KEY_FINGERPRINT = $fingerprint
$env:CN_FORM_RELAY_ORIGIN    = "http://127.0.0.1:8799"
$env:CN_FORM_VERSION         = "remote-rehearsal"
Push-Location form; npm run build; Pop-Location   # -> form/dist
```

### 3. Generate the D8 manifest over the built form and pin it

```powershell
$manifest = Join-Path $work "dist.manifest.json"
node form/scripts/gen-manifest.mjs --dist form/dist --out $manifest `
  --fingerprint $fingerprint --form-version "remote-rehearsal" --commit "local-rehearsal"
$manifestHash = (Get-FileHash -Algorithm SHA256 $manifest).Hash.ToLower()
```

### 4. Serve `form/dist` so the puller's D8 check can reach it (Verified path)

In a SEPARATE terminal - leave it running:

```powershell
npx --yes http-server form/dist -p 8080 -a 127.0.0.1 --silent
# pages_origin for the puller is then http://127.0.0.1:8080
```

### 5. Run the relay locally

The relay stores only the SHA-256 of the bearer credential, and admits only
envelopes whose recipient fingerprint is on the allowlist. In ANOTHER terminal:

```powershell
$token = "cn-relay-rehearsal-token"
$credHash = (Get-FileHash -Algorithm SHA256 -InputStream `
  ([IO.MemoryStream]::new([Text.Encoding]::UTF8.GetBytes($token)))).Hash.ToLower()
$env:WRANGLER_SEND_METRICS = "false"
Push-Location relay
node node_modules/wrangler/bin/wrangler.js dev --local --ip 127.0.0.1 --port 8799 `
  --persist-to (Join-Path $work "wrangler-state") `
  --var "CREDENTIAL_HASH:$credHash" --var "ADMISSION_ALLOWLIST:$fingerprint"
Pop-Location
```

### 6. Submit through the real form in a browser

Open `http://127.0.0.1:8080` (the served `form/dist`). Fill the fields with
SYNTHETIC data - a fictional display name, `someone@example.test`, etc. - affirm
the consent checkbox (the structural gate, D-030), and submit. The form seals
client-side (libsodium sealed box) and POSTs the ciphertext to the relay. On
success it shows a receipt id. Note it.

### 7. Prepare the puller config, pinning the REAL manifest and a reachable pages origin

Write `$work\pull-config.json` (this is the Verified counterpart to what
`scripts/e2e/prepare-pull-config.mjs` writes for the automated Degraded run):

```powershell
$token | Set-Content -NoNewline (Join-Path $work "credential.txt")
@{
  config_version = "0.1.0"
  key_dir = $keys
  key_fingerprint_pin = $fingerprint
  relay_origin = "http://127.0.0.1:8799"
  credential_path = (Join-Path $work "credential.txt")
  pages_origin = "http://127.0.0.1:8080"           # reachable -> Verified
  manifest_path = $manifest
  manifest_pin = @{ manifest_hash = $manifestHash; commit_sha = "local-rehearsal";
                    pinned_at = (Get-Date).ToString("o"); pinned_by = "e2e-remote-intake" }
  known_consent_digests = @()                        # unknown digest -> WARNING, not a halt
  blob_ttl_seconds = 86400
  max_pull_interval_seconds = 3600
} | ConvertTo-Json -Depth 8 | Set-Content (Join-Path $work "pull-config.json")
```

### 8. Pull over real HTTP

```powershell
$queue = Join-Path $work "queue"; New-Item -ItemType Directory -Force $queue | Out-Null
cargo run --manifest-path core/Cargo.toml -p cn -- intake pull `
  --config (Join-Path $work "pull-config.json") --queue $queue
# Enter the ceremony passphrase when prompted.
```

Expected in the JSON report: `preconditions.bundle_check.status = "verified"`
(the served `form/dist` matched the pinned manifest and the key fingerprint),
`main_loop.staged = 1`, `main_loop.blobs_deleted = 1`, no errors. A
`*.record.json` with `source.kind = "remote"` now sits in `$queue`.

### 9. Approve in the facilitator wizard

```powershell
Push-Location app; npm run dev; Pop-Location    # open the printed localhost URL
```

In the app: open the facilitator intake wizard, grant the queue directory
(`$queue`) via the directory picker, open the staged record in the review view,
confirm the three read-only core checks, and click Approve. The wizard writes a
create-only decision file into `$queue\decisions\`. (Shortcut for a headless run:
`cargo run --manifest-path core/Cargo.toml -p cn --example emit-approve-decision -- --queue $queue --reviewer 00000000-0000-0000-0000-0000000007d1`.)

### 10. Apply and export

`cn intake apply` is the native durable owner (I2/D4) - the only writer of the op
log. Use a fisheries fixture group + an active member as facilitator:

```powershell
$ops = Join-Path $work "ops.jsonl"
Copy-Item fixtures/groups/fisheries-committee.ops.jsonl $ops -Force
$group = "00000000-0000-0000-0000-000000000011"
$fac   = "00000000-0000-0000-0000-0000000007d1"
cargo run --manifest-path core/Cargo.toml -p cn -- intake apply `
  --queue $queue --ops $ops --group $group --facilitator $fac --kind person
cargo run --manifest-path core/Cargo.toml -p cn -- export `
  --template fixtures/templates/fisheries-committee.template.json `
  --ops $ops --group $group --viewer "person:$fac"
```

The submission you typed in step 6 appears in the exported, permission-filtered
graph. That is the pass condition.

## Expected-results checklist

- [ ] `/submit` returned a receipt id (not 409 `form_out_of_date` - a 409 means
      the built form's fingerprint is not on the relay allowlist).
- [ ] Pull report: `bundle_check.status = "verified"`, `staged = 1`,
      `blobs_deleted = 1`, `errors = []`.
- [ ] Staged record `source.kind = "remote"` with the receipt id from step 6.
- [ ] `intake apply`: `ops_appended = 1`, decision `admitted`, transaction
      `intent_completed`, `review_states.approved = 1`.
- [ ] `export`: the display name you typed is present.

## Cleanup

Stop the served-form and relay terminals. Then:

```powershell
Remove-Item -Recurse -Force $work
Remove-Item Env:\CN_FORM_PUBLIC_KEY_HEX, Env:\CN_FORM_KEY_FINGERPRINT, `
  Env:\CN_FORM_RELAY_ORIGIN, Env:\CN_FORM_VERSION -ErrorAction SilentlyContinue
```

## Notes

- **Verified vs Degraded (D8).** This runbook serves `form/dist` so the puller
  reads and hash-matches the full bundle against the pinned manifest -> Verified.
  The automated script points `pages_origin` at a closed port so the same code
  path lands Degraded (proceeds on key-pin alone). Both are ADR-005 D8 compliant;
  only the reachability differs.
- **Not the deploy bar.** Real deployment (Pages + Workers, real ceremony key,
  D-023-signed form text) is `docs/runbooks/intake-relay-deploy.md` (blueprint
  step 10), gated by D-059.8. Nothing here touches that gate.
