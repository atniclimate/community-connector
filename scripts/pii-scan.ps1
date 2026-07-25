#Requires -Version 7
<#
PII tripwire for Community Navigator (AGENTS.md I1).

Blocks:
  - email addresses outside the @example.test fixture namespace, unless the
    exact address is listed in scripts/pii-allowlist.txt (operator identities
    only - never partner or community-member addresses)
  - phone-number patterns
  - red-data path patterns inherited from the predecessor exclusion list
  - intake-queue tripwires (ADR-005 D4, blueprint section 7): queue-shaped
    file names (*.record.json, *.sidecar.json, *.reviewed) anywhere, the
    "queue_record_version" JSON data marker, and the "secret-encrypted"
    key-envelope marker. These are TRIPWIRES under the I1 process boundary,
    not enforcement: a stripped-marker copy or bypassed hook defeats them
    (ADR-005 D4 honest-description rule). Rust/TypeScript sources are exempt
    from the CONTENT markers only - they name the format keys as
    identifiers; a real staged queue file would never arrive as source.

Usage:
  pwsh scripts/pii-scan.ps1            # scan working tree (tracked + untracked)
  pwsh scripts/pii-scan.ps1 -Staged    # scan staged changes (pre-commit mode)
  pwsh scripts/pii-scan.ps1 -SelfTest  # generate marker-bearing fixtures in
                                       # the session temp dir, assert each is
                                       # flagged, clean up (proves the
                                       # tripwires live without ever
                                       # committing anything marker-shaped)

Exit code 0 = clean (or self-test passed), 1 = violations found (commit is
blocked in hook mode) or self-test failure. False positives get a specific
allowlist entry with a comment, never a rule relaxation.
#>
param([switch]$Staged, [switch]$SelfTest)

$ErrorActionPreference = 'Stop'

$emailRx     = '[A-Za-z0-9._%+\-]+@[A-Za-z0-9.\-]+\.[A-Za-z]{2,}'
$phoneRx     = '\(?\b\d{3}\)?[-. ]\d{3}[-. ]\d{4}\b'
$redPathRx   = '(?i)(^|[\\/])(source_data|research_edges|t1_partners)([\\/]|$)|(?i)_partners|(?i)participants'
$queuePathRx = '(?i)\.(record|sidecar)\.json$|\.reviewed$'
$queueDataRx = '"queue_record_version"\s*:'
$secretRx    = 'secret-encrypted'
# Sources naming format keys as identifiers, and docs DESCRIBING the
# tripwires (content markers only; path rules still apply to everything -
# a real queue file never arrives as source or markdown).
$markerContentExemptExt = @('.rs', '.ts', '.md')

# Binary or generated formats never scanned for content (paths still checked).
$skipContentExt = @('.png','.jpg','.jpeg','.gif','.ico','.woff','.woff2','.ttf',
                    '.otf','.wasm','.zip','.pdf')
$skipContentNames = @('package-lock.json','Cargo.lock','pii-allowlist.txt','pii-scan.ps1')

function Test-FileViolations {
    param([string]$RelPath, [string]$Content, [string[]]$Allowed)
    $found = [System.Collections.Generic.List[string]]::new()

    if ($RelPath -match $redPathRx) {
        $found.Add("RED PATH   $RelPath  (matches predecessor exclusion pattern)")
        return $found
    }
    if ($RelPath -match $queuePathRx) {
        $found.Add("QUEUE PATH $RelPath  (intake-queue file shape; queue data never enters the repo, ADR-005 D4)")
        return $found
    }

    $ext  = [System.IO.Path]::GetExtension($RelPath).ToLowerInvariant()
    $name = [System.IO.Path]::GetFileName($RelPath)
    if ($skipContentExt -contains $ext) { return $found }
    if ($skipContentNames -contains $name) { return $found }
    if (-not $Content) { return $found }

    foreach ($m in [regex]::Matches($Content, $emailRx)) {
        $addr = $m.Value.ToLowerInvariant()
        if ($addr.EndsWith('@example.test')) { continue }
        if ($Allowed -contains $addr) { continue }
        $found.Add("EMAIL      $RelPath  contains '$($m.Value)' (only @example.test or allowlisted operator addresses permitted)")
    }
    foreach ($m in [regex]::Matches($Content, $phoneRx)) {
        $found.Add("PHONE      $RelPath  contains phone-like pattern '$($m.Value)'")
    }
    if ($markerContentExemptExt -notcontains $ext) {
        if ($Content -match $queueDataRx) {
            $found.Add("QUEUE DATA $RelPath  carries the queue_record_version data marker (staged intake data never enters the repo, ADR-005 D4)")
        }
        if ($Content -match $secretRx) {
            $found.Add("KEY MATTER $RelPath  carries the secret-encrypted key-envelope marker (key material never enters the repo)")
        }
    }
    return $found
}

if ($SelfTest) {
    # Positive fixtures are GENERATED here at runtime and removed after:
    # nothing marker-shaped is ever committed, so the tripwire is proven
    # live without tripping itself (blueprint section 7).
    $tempRoot = Join-Path ([System.IO.Path]::GetTempPath()) "cn-pii-selftest-$([guid]::NewGuid())"
    New-Item -ItemType Directory -Force $tempRoot | Out-Null
    try {
        $cases = @(
            @{ Name = 'fake.record.json';  Content = '{}';                                        Expect = 'QUEUE PATH' },
            @{ Name = 'fake.sidecar.json'; Content = '{}';                                        Expect = 'QUEUE PATH' },
            @{ Name = 'fake.reviewed';     Content = '';                                          Expect = 'QUEUE PATH' },
            @{ Name = 'payload.json';      Content = '{ "queue_record_version": "9.9.9" }';       Expect = 'QUEUE DATA' },
            @{ Name = 'envelope.txt';      Content = 'age header: secret-encrypted key follows';  Expect = 'KEY MATTER' },
            @{ Name = 'contact.md';        Content = 'reach me at realperson@somewhere.org';      Expect = 'EMAIL' }
        )
        $failures = 0
        foreach ($case in $cases) {
            $path = Join-Path $tempRoot $case.Name
            Set-Content -Path $path -Value $case.Content -NoNewline
            $hits = Test-FileViolations -RelPath $case.Name -Content $case.Content -Allowed @()
            $matched = @($hits | Where-Object { $_.StartsWith($case.Expect) })
            if ($matched.Count -eq 0) {
                Write-Host "SELF-TEST FAIL: '$($case.Name)' did not trip the $($case.Expect) rule" -ForegroundColor Red
                $failures++
            }
        }
        # A clean source file naming the key as an identifier must NOT trip.
        $benign = Test-FileViolations -RelPath 'record.rs' -Content 'pub queue_record_version: semver::Version,' -Allowed @()
        if (@($benign).Count -ne 0) {
            Write-Host "SELF-TEST FAIL: exempt source tripped: $benign" -ForegroundColor Red
            $failures++
        }
        if ($failures -gt 0) { exit 1 }
        Write-Host "PII self-test passed: $($cases.Count) positive fixture(s) flagged, exemption holds, fixtures removed."
        exit 0
    } finally {
        Remove-Item -Recurse -Force $tempRoot -ErrorAction SilentlyContinue
    }
}

$repoRoot = (& git rev-parse --show-toplevel).Trim()

$allowlistFile = 'scripts/pii-allowlist.txt'
$allowed = @()
$allowPath = Join-Path $repoRoot $allowlistFile
if (Test-Path $allowPath) {
    $allowed = @(Get-Content $allowPath |
        Where-Object { $_.Trim() -and -not $_.Trim().StartsWith('#') } |
        ForEach-Object { $_.Trim().ToLowerInvariant() })
}

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

    $content = $null
    $ext = [System.IO.Path]::GetExtension($f).ToLowerInvariant()
    if ($skipContentExt -notcontains $ext) {
        if ($Staged) {
            $content = (& git -C $repoRoot show ":$f" 2>$null) -join "`n"
            if ($LASTEXITCODE -ne 0) { $content = $null }
        } else {
            $full = Join-Path $repoRoot $f
            if (Test-Path $full -PathType Leaf) {
                $content = Get-Content -Raw -ErrorAction SilentlyContinue $full
            }
        }
    }
    foreach ($v in (Test-FileViolations -RelPath $f -Content $content -Allowed $allowed)) {
        $violations.Add($v)
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
