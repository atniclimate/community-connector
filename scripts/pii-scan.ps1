#Requires -Version 7
<#
PII tripwire for Community Navigator (AGENTS.md I1).

Blocks:
  - email addresses outside the @example.test fixture namespace, unless the
    exact address is listed in scripts/pii-allowlist.txt (operator identities
    only - never partner or community-member addresses)
  - phone-number patterns
  - red-data path patterns inherited from the predecessor exclusion list

Usage:
  pwsh scripts/pii-scan.ps1           # scan working tree (tracked + untracked)
  pwsh scripts/pii-scan.ps1 -Staged   # scan staged changes (pre-commit mode)

Exit code 0 = clean, 1 = violations found (commit is blocked in hook mode).
False positives get a specific allowlist entry with a comment, never a rule
relaxation.
#>
param([switch]$Staged)

$ErrorActionPreference = 'Stop'
$repoRoot = (& git rev-parse --show-toplevel).Trim()

$allowlistFile = 'scripts/pii-allowlist.txt'
$allowed = @()
$allowPath = Join-Path $repoRoot $allowlistFile
if (Test-Path $allowPath) {
    $allowed = @(Get-Content $allowPath |
        Where-Object { $_.Trim() -and -not $_.Trim().StartsWith('#') } |
        ForEach-Object { $_.Trim().ToLowerInvariant() })
}

$emailRx   = '[A-Za-z0-9._%+\-]+@[A-Za-z0-9.\-]+\.[A-Za-z]{2,}'
$phoneRx   = '\(?\b\d{3}\)?[-. ]\d{3}[-. ]\d{4}\b'
$redPathRx = '(?i)(^|[\\/])(source_data|research_edges|t1_partners)([\\/]|$)|(?i)_partners|(?i)participants'

# Binary or generated formats never scanned for content (paths still checked).
$skipContentExt = @('.png','.jpg','.jpeg','.gif','.ico','.woff','.woff2','.ttf',
                    '.otf','.wasm','.zip','.pdf')
$skipContentNames = @('package-lock.json','Cargo.lock','pii-allowlist.txt')

if ($Staged) {
    $files = @(& git -C $repoRoot diff --cached --name-only --diff-filter=ACMR)
} else {
    $tracked   = @(& git -C $repoRoot ls-files)
    $untracked = @(& git -C $repoRoot ls-files --others --exclude-standard)
    $files = @($tracked + $untracked | Sort-Object -Unique)
}

$violations = [System.Collections.Generic.List[string]]::new()

foreach ($f in $files) {
    if (-not $f) { continue }

    if ($f -match $redPathRx) {
        $violations.Add("RED PATH   $f  (matches predecessor exclusion pattern)")
        continue
    }

    $ext  = [System.IO.Path]::GetExtension($f).ToLowerInvariant()
    $name = [System.IO.Path]::GetFileName($f)
    if ($skipContentExt -contains $ext) { continue }
    if ($skipContentNames -contains $name) { continue }

    if ($Staged) {
        $content = (& git -C $repoRoot show ":$f" 2>$null) -join "`n"
        if ($LASTEXITCODE -ne 0) { continue }
    } else {
        $full = Join-Path $repoRoot $f
        if (-not (Test-Path $full -PathType Leaf)) { continue }
        $content = Get-Content -Raw -ErrorAction SilentlyContinue $full
    }
    if (-not $content) { continue }

    foreach ($m in [regex]::Matches($content, $emailRx)) {
        $addr = $m.Value.ToLowerInvariant()
        if ($addr.EndsWith('@example.test')) { continue }
        if ($allowed -contains $addr) { continue }
        $violations.Add("EMAIL      $f  contains '$($m.Value)' (only @example.test or allowlisted operator addresses permitted)")
    }
    foreach ($m in [regex]::Matches($content, $phoneRx)) {
        $violations.Add("PHONE      $f  contains phone-like pattern '$($m.Value)'")
    }
}

if ($violations.Count -gt 0) {
    Write-Host "PII SCAN FAILED - $($violations.Count) violation(s):" -ForegroundColor Red
    $violations | Sort-Object -Unique | ForEach-Object { Write-Host "  $_" -ForegroundColor Red }
    Write-Host "Fix the content, or for a confirmed false positive add a commented entry to $allowlistFile."
    exit 1
}

Write-Host "PII scan clean ($($files.Count) file(s) checked$(if ($Staged) { ', staged mode' }))."
exit 0
