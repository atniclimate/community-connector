<#
.SYNOPSIS
  Automated, opt-in form-to-graph rehearsal for the REMOTE intake pipeline on
  SYNTHETIC data (blueprint docs/blueprints/intake-relay.md section 8 step 9,
  Phase E; ADR-005 D1/D3/D4/D6/D8).

.DESCRIPTION
  Drives the whole remote pipeline end to end, with NO browser, and FAILS LOUDLY
  if the synthetic entity does not reach the permission-filtered graph:

    keygen (ceremony key)                 cn intake keygen
    -> seal a synthetic submission        scripts/e2e/seal-submission.mjs (real form crypto)
    -> run the REAL Worker relay locally  wrangler dev (miniflare, local KV)
    -> POST the sealed envelope           /submit  (200 + receipt_id)
    -> pull over REAL HTTP (ureq)         cn intake pull  (stages a Remote record, deletes the blob)
    -> facilitator approves               emit-approve-decision example -> decisions/
    -> apply (native durable owner)       cn intake apply  (appends the EntityCreate)
    -> reload + export the graph          cn export --viewer person:<facilitator>
    -> assert the display_name is visible

  This is NOT a check-all member. Like scripts/generate-crypto-vectors.js it
  needs Node (libsodium) and a running Worker, so it is opt-in and run by hand
  (blueprint section 1 "no Node dependency in CI"). It is ADDITIVE: it modifies
  no shipped logic and touches nothing under git - keys, queue, relay KV state,
  and the sealed envelope all live in a system temp dir removed on exit.

  PII posture (I1): the demo member is fictional and every email is
  @example.test. Nothing synthetic is ever committed.

  Bundle check (a step-9 design point): the automated run points pages_origin at
  a closed local port, so the puller's D8 verify_bundle runs for real (reads +
  hash-pins the local manifest, matches the embedded key fingerprint) and lands
  Degraded on the unreachable origin (proceeds on key-pin only). The Verified
  path against a served real form/dist is the manual runbook's job
  (docs/runbooks/e2e-remote-intake.md).

.EXAMPLE
  pwsh scripts/e2e-remote-intake.ps1
  pwsh scripts/e2e-remote-intake.ps1 -Port 8791 -KeepArtifacts
#>
[CmdletBinding()]
param(
  # Localhost port for the local Worker relay (miniflare).
  [int]$Port = 8799,
  # Keep the temp workspace (keys, queue, relay state, envelope, snapshot) for
  # debugging instead of deleting it on exit.
  [switch]$KeepArtifacts
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

$RepoRoot = Split-Path -Parent $PSScriptRoot
$CoreManifest = Join-Path $RepoRoot "core/Cargo.toml"
$RelayDir = Join-Path $RepoRoot "relay"
$WranglerJs = Join-Path $RelayDir "node_modules/wrangler/bin/wrangler.js"
$Exe = if ($IsWindows) { ".exe" } else { "" }
$CnBin = Join-Path $RepoRoot "core/target/debug/cn$Exe"
$EmitBin = Join-Path $RepoRoot "core/target/debug/examples/emit-approve-decision$Exe"
$Template = Join-Path $RepoRoot "fixtures/templates/fisheries-committee.template.json"
$FixtureOps = Join-Path $RepoRoot "fixtures/groups/fisheries-committee.ops.jsonl"

# --- Synthetic constants (I1) ------------------------------------------------
# The fisheries fixture group id, and one of its ACTIVE members (governance
# role). `cn intake apply` authorizes an UNOWNED intake create with require_member
# (cn-perm authz.rs), so any active member is a valid facilitator here.
$GroupId = "00000000-0000-0000-0000-000000000011"
$Facilitator = "00000000-0000-0000-0000-0000000007d1"
# A >= 6-word passphrase for the throwaway ceremony key (keymat.rs minimum).
$Passphrase = "east river fisheries pilot rehearsal ceremony passphrase"
# A synthetic control-plane bearer token; the relay stores only its SHA-256.
$BearerToken = "cn-relay-e2e-local-token"
# A synthetic 64-hex consent-text digest, listed as "known" so the pull warns
# about nothing (the value is opaque to the pipeline).
$ConsentDigest = "e2e0" * 16

$base = "http://127.0.0.1:$Port"
$pagesClosed = "http://127.0.0.1:1"   # a closed port -> D8 Degraded (proceeds)

function Fail([string]$msg) { throw "E2E FAILED: $msg" }
function Step([string]$msg) { Write-Host "==> $msg" -ForegroundColor Cyan }
function Ok([string]$msg) { Write-Host "    OK: $msg" -ForegroundColor Green }

# --- Temp workspace (OUTSIDE the git worktree) -------------------------------
$Work = Join-Path ([System.IO.Path]::GetTempPath()) ("cn-e2e-" + [System.Guid]::NewGuid().ToString("N"))
$Keys = Join-Path $Work "keys"
$Queue = Join-Path $Work "queue"
$State = Join-Path $Work "wrangler-state"
$Envelope = Join-Path $Work "envelope.json"
$PullConfig = Join-Path $Work "pull-config.json"
$Manifest = Join-Path $Work "dist.manifest.json"
$Credential = Join-Path $Work "credential.txt"
$Ops = Join-Path $Work "ops.jsonl"
$Snapshot = Join-Path $Work "snapshot.json"
$KeygenLog = Join-Path $Work "keygen.stderr.log"
$WranglerLog = Join-Path $Work "wrangler.stdout.log"
$WranglerErr = Join-Path $Work "wrangler.stderr.log"
$PullErr = Join-Path $Work "pull.stderr.log"
$ApplyErr = Join-Path $Work "apply.stderr.log"
$ExportErr = Join-Path $Work "export.stderr.log"
New-Item -ItemType Directory -Force -Path $Work, $Queue, $State | Out-Null

$wrangler = $null

function Stop-Relay {
  if ($null -ne $script:wrangler -and -not $script:wrangler.HasExited) {
    # Kill the whole tree: node -> workerd. /T covers children, /F forces.
    & taskkill /PID $script:wrangler.Id /T /F 2>$null | Out-Null
  }
  # Belt and suspenders: kill whoever still holds the port (workerd can outlive
  # a detached parent).
  try {
    Get-NetTCPConnection -LocalPort $Port -State Listen -ErrorAction SilentlyContinue |
      Select-Object -ExpandProperty OwningProcess -Unique |
      ForEach-Object { & taskkill /PID $_ /T /F 2>$null | Out-Null }
  } catch { }
}

try {
  Write-Host ""
  Write-Host "Remote-intake form-to-graph rehearsal (synthetic data, no browser)" -ForegroundColor White
  Write-Host "Workspace: $Work" -ForegroundColor DarkGray
  Write-Host ""

  # 0. Build the CLI and the opt-in approve-decision example.
  Step "cargo build (cn + emit-approve-decision example)"
  & cargo build --quiet --manifest-path $CoreManifest -p cn
  if ($LASTEXITCODE -ne 0) { Fail "cargo build -p cn failed" }
  & cargo build --quiet --manifest-path $CoreManifest -p cn --example emit-approve-decision
  if ($LASTEXITCODE -ne 0) { Fail "cargo build --example emit-approve-decision failed" }
  if (-not (Test-Path $CnBin)) { Fail "cn binary not found at $CnBin" }
  if (-not (Test-Path $EmitBin)) { Fail "example binary not found at $EmitBin" }
  Ok "binaries built"

  # 1. Ceremony key (passphrase piped: keygen reads passphrase + confirmation).
  Step "cn intake keygen (throwaway ceremony key)"
  @($Passphrase, $Passphrase) | & $CnBin intake keygen --out $Keys 2>$KeygenLog | Out-Null
  if ($LASTEXITCODE -ne 0) { Get-Content $KeygenLog | Write-Host; Fail "keygen failed" }
  $pub = Get-Content (Join-Path $Keys "public.json") -Raw | ConvertFrom-Json
  $Fingerprint = $pub.fingerprint
  if ([string]::IsNullOrWhiteSpace($Fingerprint)) { Fail "no fingerprint in public.json" }
  Ok "key fingerprint $Fingerprint"

  # 2. Seal a synthetic submission via the form's REAL crypto path.
  Step "seal a synthetic submission (real form crypto.ts)"
  $sealJson = & node (Join-Path $RepoRoot "scripts/e2e/seal-submission.mjs") `
    --public (Join-Path $Keys "public.json") --out $Envelope --digest $ConsentDigest
  if ($LASTEXITCODE -ne 0) { Fail "seal-submission.mjs failed" }
  $seal = $sealJson | ConvertFrom-Json
  if ($seal.recipient_fingerprint -ne $Fingerprint) { Fail "sealed envelope fingerprint mismatch" }
  $DisplayName = $seal.display_name
  $SubmissionId = $seal.submission_id
  Ok "sealed submission '$DisplayName' ($($seal.envelope_bytes) bytes)"

  # 3. Prepare the puller config + D8 manifest + credential file.
  Step "prepare pull config, D8 manifest, credential"
  & node (Join-Path $RepoRoot "scripts/e2e/prepare-pull-config.mjs") `
    --keydir $Keys --fingerprint $Fingerprint --relay $base --pages $pagesClosed `
    --digest $ConsentDigest --token $BearerToken `
    --manifest $Manifest --config $PullConfig --credential $Credential | Out-Null
  if ($LASTEXITCODE -ne 0) { Fail "prepare-pull-config.mjs failed" }
  Ok "pull config written"

  # 4. Run the REAL Worker relay locally (miniflare, local KV, --var secrets).
  Step "start local Worker relay (wrangler dev on $base)"
  $sha = [System.Security.Cryptography.SHA256]::Create()
  $CredentialHash = ([System.BitConverter]::ToString(
      $sha.ComputeHash([System.Text.Encoding]::UTF8.GetBytes($BearerToken))) -replace '-', '').ToLower()
  $env:WRANGLER_SEND_METRICS = "false"
  $env:CI = "1"
  $wrArgs = @(
    $WranglerJs, "dev", "--local", "--ip", "127.0.0.1", "--port", "$Port",
    "--persist-to", $State,
    "--var", "CREDENTIAL_HASH:$CredentialHash",
    "--var", "ADMISSION_ALLOWLIST:$Fingerprint"
  )
  $script:wrangler = Start-Process -FilePath "node" -ArgumentList $wrArgs -WorkingDirectory $RelayDir `
    -PassThru -WindowStyle Hidden -RedirectStandardOutput $WranglerLog -RedirectStandardError $WranglerErr

  # Poll readiness: any HTTP answer (a 404 without auth) means the Worker is up.
  $ready = $false
  for ($i = 0; $i -lt 60; $i++) {
    if ($script:wrangler.HasExited) {
      Get-Content $WranglerErr -ErrorAction SilentlyContinue | Write-Host
      Fail "wrangler dev exited during startup (code $($script:wrangler.ExitCode))"
    }
    try {
      Invoke-WebRequest -Uri "$base/receipts" -Method GET -SkipHttpErrorCheck -TimeoutSec 3 | Out-Null
      $ready = $true; break
    } catch { Start-Sleep -Seconds 1 }
  }
  if (-not $ready) { Fail "relay did not become ready on $base" }
  Ok "relay ready (pid $($script:wrangler.Id))"

  # 5. Submit the sealed envelope (off-allowlist -> 409; here it must be 200).
  Step "POST /submit the sealed envelope"
  $envBody = Get-Content $Envelope -Raw
  $submit = Invoke-WebRequest -Uri "$base/submit" -Method POST -ContentType "application/json" `
    -Body $envBody -SkipHttpErrorCheck
  if ($submit.StatusCode -eq 409) {
    Fail "relay returned 409 form_out_of_date - the keygen fingerprint is not on the admission allowlist (wiring bug)"
  }
  if ($submit.StatusCode -ne 200) { Fail "POST /submit returned $($submit.StatusCode)" }
  $ReceiptId = ($submit.Content | ConvertFrom-Json).receipt_id
  if ([string]::IsNullOrWhiteSpace($ReceiptId)) { Fail "no receipt_id in the /submit response" }
  Ok "accepted; receipt_id $ReceiptId"

  # 6. Pull over the REAL ureq HTTP path (passphrase piped).
  Step "cn intake pull (real HTTP: receipts -> blob -> decrypt -> stage -> delete)"
  $pullOut = $Passphrase | & $CnBin intake pull --config $PullConfig --queue $Queue 2>$PullErr
  $pullExit = $LASTEXITCODE
  $pullReport = ($pullOut -join "`n") | ConvertFrom-Json
  if ($pullExit -ne 0) {
    Get-Content $PullErr -ErrorAction SilentlyContinue | Write-Host
    Fail "pull exited $pullExit`n$($pullReport | ConvertTo-Json -Depth 8)"
  }
  if ($pullReport.preconditions.bundle_check.status -ne "degraded") {
    Fail "expected Degraded bundle (unreachable pages origin), got '$($pullReport.preconditions.bundle_check.status)'"
  }
  if ($pullReport.main_loop.staged -ne 1) { Fail "expected staged=1, got $($pullReport.main_loop.staged)" }
  if ($pullReport.main_loop.blobs_deleted -ne 1) { Fail "expected blobs_deleted=1, got $($pullReport.main_loop.blobs_deleted)" }
  if ($pullReport.main_loop.errors.Count -ne 0) { Fail "pull reported errors: $($pullReport.main_loop.errors -join '; ')" }
  Ok "pull staged 1, deleted 1 blob, bundle Degraded, no errors"

  # 6b. The staged record carries the Remote source with the right receipt.
  # @() forces an array so .Count is valid under StrictMode even for one match.
  $recordFile = @(Get-ChildItem $Queue -Filter *.record.json)
  if ($recordFile.Count -ne 1) { Fail "expected exactly one staged record, found $($recordFile.Count)" }
  $rec = Get-Content $recordFile[0].FullName -Raw | ConvertFrom-Json
  if ($rec.source.kind -ne "remote") { Fail "staged record source is not remote" }
  if ($rec.source.receipt_id -ne $ReceiptId) { Fail "staged record receipt_id mismatch" }
  if ($rec.source.key_used -ne $Fingerprint) { Fail "staged record key_used mismatch" }
  if ($rec.payload.submission_id -ne $SubmissionId) { Fail "staged record submission_id mismatch" }
  Ok "staged record is SubmissionSource::Remote (receipt $ReceiptId, key $Fingerprint)"

  # 7. Facilitator approval: emit an approve decision into the queue inbox.
  Step "emit approve decision (facilitator $Facilitator)"
  $emitOut = & $EmitBin --queue $Queue --reviewer $Facilitator
  if ($LASTEXITCODE -ne 0) { Fail "emit-approve-decision failed" }
  $emit = $emitOut | ConvertFrom-Json
  Ok "approve decision $($emit.decision_id) bound to record $($emit.record_id)"

  # 8. Apply: the native durable owner appends the EntityCreate (I2/D4).
  Step "cn intake apply (native durable owner)"
  Copy-Item $FixtureOps $Ops -Force
  $applyOut = & $CnBin intake apply --queue $Queue --ops $Ops --group $GroupId `
    --facilitator $Facilitator --kind person 2>$ApplyErr
  $applyExit = $LASTEXITCODE
  $applyReport = ($applyOut -join "`n") | ConvertFrom-Json
  if ($applyExit -ne 0) {
    Get-Content $ApplyErr -ErrorAction SilentlyContinue | Write-Host
    Fail "apply exited $applyExit`n$($applyReport | ConvertTo-Json -Depth 8)"
  }
  if ($applyReport.ops_appended -ne 1) {
    Write-Host ($applyReport | ConvertTo-Json -Depth 10)
    Fail "expected ops_appended=1, got $($applyReport.ops_appended)"
  }
  $decisions = @($applyReport.decisions)
  if ($decisions.Count -lt 1) { Fail "apply consumed no decisions" }
  if ($decisions[0].outcome -ne "admitted") { Fail "decision not admitted: $($decisions[0].outcome)" }
  if ($decisions[0].transaction.kind -ne "intent_completed") {
    Fail "transaction not intent_completed: $($decisions[0].transaction.kind)"
  }
  if ($applyReport.review_states.approved -ne 1) { Fail "expected review_states.approved=1" }
  Ok "apply appended 1 EntityCreate; record approved"

  # 9. Reload + export the permission-filtered graph and assert visibility.
  Step "cn export (reload -> permission-filtered projection)"
  $exportOut = & $CnBin export --template $Template --ops $Ops --group $GroupId `
    --viewer "person:$Facilitator" 2>$ExportErr
  if ($LASTEXITCODE -ne 0) {
    Get-Content $ExportErr -ErrorAction SilentlyContinue | Write-Host
    Fail "export failed"
  }
  $snapshotText = $exportOut -join "`n"
  Set-Content -Path $Snapshot -Value $snapshotText
  if (-not $snapshotText.Contains($DisplayName)) {
    Fail "the approved submission '$DisplayName' is NOT in the exported graph"
  }
  Ok "'$DisplayName' is visible in the permission-filtered graph"

  Write-Host ""
  Write-Host "E2E PASSED: synthetic remote submission reached the graph end to end." -ForegroundColor Green
  Write-Host "  seal -> POST /submit -> pull(real HTTP) -> approve -> apply -> export" -ForegroundColor Green
  Write-Host ""
  $exitCode = 0
}
catch {
  Write-Host ""
  Write-Host $_.Exception.Message -ForegroundColor Red
  Write-Host ""
  $exitCode = 1
}
finally {
  Stop-Relay
  if ($KeepArtifacts) {
    Write-Host "Artifacts kept at: $Work" -ForegroundColor Yellow
  } else {
    Remove-Item -Recurse -Force $Work -ErrorAction SilentlyContinue
  }
}

exit $exitCode
