<#
.SYNOPSIS
  Build the remote intake form (form/) into its deployable artifact and emit the
  ADR-005 D8 canonical deploy manifest. Relay blueprint step 6.

.DESCRIPTION
  Orchestrates: read the group template + the embedded public key/fingerprint
  (from the committed TEST-ONLY defaults, or overrides passed here) -> npm build
  (Vite) -> generate the D8 manifest with provenance.

  This BUILDS a deployable artifact; it does NOT deploy. The D-059.8 deploy bar
  (intake pipeline working + keygen ceremony executed + D-023 sign-off) is
  unmet. The dist/ output and the manifest are gitignored and never committed
  (D8: dist/ deploys directly to Pages; the manifest is pinned off-repo).

  Defaults use the synthetic TEST-ONLY keypair from
  fixtures/crypto/sealed-box-vectors.json - NEVER an operational key. The real
  facilitator key is produced by the (out-of-scope) keygen ceremony and passed
  via -PublicKeyHex / -Fingerprint. These defaults MUST match the defaults in
  form/vite.config.ts, which bakes the same values into the bundle.

.EXAMPLE
  pwsh scripts/build-form.ps1
  pwsh scripts/build-form.ps1 -CheckReproducible
  pwsh scripts/build-form.ps1 -RelayOrigin https://relay.example.test -CheckReproducible
#>
[CmdletBinding()]
param(
  [string]$PublicKeyHex = "205fd0dce7b409a3231b86dad10f6e3a276bb2e0838bc605501131f96b117a71",
  [string]$Fingerprint  = "efdf-7ce7-69fa-feeb-7512-a100-6450-45c6",
  [string]$RelayOrigin  = "http://localhost:8787",
  [string]$FormVersion  = "remote-draft-2026-08-11",
  [string]$TemplatePath = "",
  [string]$Builder      = "local-build",
  [string]$ManifestOut  = "",
  [switch]$CheckReproducible
)

$ErrorActionPreference = "Stop"
$RepoRoot = Split-Path -Parent $PSScriptRoot
$FormDir  = Join-Path $RepoRoot "form"
if ($ManifestOut -eq "") { $ManifestOut = Join-Path $FormDir "dist.manifest.json" }
if ($TemplatePath -eq "") {
  $TemplatePath = Join-Path $RepoRoot "fixtures/templates/research-network.template.json"
}

# Build-time config for Vite's `define` (process-scoped; not persisted).
$env:CN_FORM_PUBLIC_KEY_HEX = $PublicKeyHex
$env:CN_FORM_KEY_FINGERPRINT = $Fingerprint
$env:CN_FORM_RELAY_ORIGIN = $RelayOrigin
$env:CN_FORM_VERSION = $FormVersion
$env:CN_FORM_TEMPLATE_PATH = $TemplatePath

$commit = (& git -C $RepoRoot rev-parse HEAD).Trim()

function Invoke-FormBuild {
  Write-Host "==> npm build (Vite) ..."
  if (Test-Path (Join-Path $FormDir "package-lock.json")) {
    & npm --prefix $FormDir ci
  } else {
    & npm --prefix $FormDir install
  }
  if ($LASTEXITCODE -ne 0) { throw "npm install/ci failed" }
  & npm --prefix $FormDir run build
  if ($LASTEXITCODE -ne 0) { throw "vite build failed" }
}

function Invoke-Manifest([string]$OutPath) {
  & node (Join-Path $FormDir "scripts/gen-manifest.mjs") `
    --dist (Join-Path $FormDir "dist") `
    --out $OutPath `
    --commit $commit `
    --form-version $FormVersion `
    --fingerprint $Fingerprint `
    --builder $Builder
  if ($LASTEXITCODE -ne 0) { throw "manifest generation failed" }
}

Invoke-FormBuild
Invoke-Manifest $ManifestOut

if ($CheckReproducible) {
  Write-Host "==> reproducibility check: second build, diffing deployed file bytes ..."
  $tmp = Join-Path ([System.IO.Path]::GetTempPath()) ("cn-form-manifest2-" + [System.Guid]::NewGuid().ToString("N") + ".json")
  try {
    Invoke-FormBuild
    Invoke-Manifest $tmp
    $files1 = (Get-Content $ManifestOut -Raw | ConvertFrom-Json).files | ConvertTo-Json -Depth 6
    $files2 = (Get-Content $tmp -Raw | ConvertFrom-Json).files | ConvertTo-Json -Depth 6
    if ($files1 -eq $files2) {
      Write-Host "REPRODUCIBLE: deployed file set is byte-identical across two builds." -ForegroundColor Green
    } else {
      Write-Error "NOT REPRODUCIBLE: deployed file hashes differ across two builds."
      exit 1
    }
  } finally {
    if (Test-Path $tmp) { Remove-Item $tmp -Force }
  }
}

Write-Host ""
Write-Host "Build complete. Deploy set: $(Join-Path $FormDir 'dist')  (gitignored, NOT committed)."
Write-Host "Manifest: $ManifestOut  (gitignored; pin its hash off-repo at deploy per D8)."
Write-Host "DEPLOY BAR (D-059.8) is unmet: this artifact is built, not deployed."
