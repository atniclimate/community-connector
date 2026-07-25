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
    (ADR-005 D4 honest-description rule). Content-marker exemptions are
    NARROW (round-1 F12): .rs/.ts sources (they name the format keys as
    identifiers) and the EXPLICITLY LISTED docs that describe the markers -
    never all markdown. Path rules apply to every file.

Usage:
  pwsh scripts/pii-scan.ps1            # scan working tree (tracked + untracked)
  pwsh scripts/pii-scan.ps1 -Staged    # scan staged changes (pre-commit mode)
  pwsh scripts/pii-scan.ps1 -SelfTest  # write marker-bearing fixtures into the
                                       # session temp dir and run the REAL scan
                                       # loop over them (round-1 F12), assert
                                       # every rule trips, the exemption
                                       # negative holds, and an unreadable
                                       # file FAILS CLOSED (round-2 F12)

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
# Sources naming format keys as identifiers (content markers only; path
# rules still apply). Never a blanket doc exemption.
$markerContentExemptExt = @('.rs', '.ts')
# The ONLY docs allowed to carry the marker strings, because they define
# or record the tripwires themselves (round-1 F12). Any other file - any
# other markdown included - trips.
$markerContentExemptFiles = @(
    'DECISIONS.md',
    'HANDOFF.md',
    'docs/adr/ADR-005-remote-intake.md',
    'docs/blueprints/intake-pipeline.md',
    'docs/design/facilitator-keygen-ceremony.md'
)

# Binary or generated formats never scanned for content (paths still checked).
$skipContentExt = @('.png','.jpg','.jpeg','.gif','.ico','.woff','.woff2','.ttf',
                    '.otf','.wasm','.zip','.pdf')
$skipContentNames = @('package-lock.json','Cargo.lock','pii-allowlist.txt','pii-scan.ps1')

function Test-FileViolations {
    param([string]$RelPath, [string]$Content, [string[]]$Allowed)
    $found = [System.Collections.Generic.List[string]]::new()
    $normalized = $RelPath -replace '\\', '/'

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
    $markerExempt = ($markerContentExemptExt -contains $ext) -or
                    ($markerContentExemptFiles -contains $normalized)
    if (-not $markerExempt) {
        if ($Content -match $queueDataRx) {
            $found.Add("QUEUE DATA $RelPath  carries the queue_record_version data marker (staged intake data never enters the repo, ADR-005 D4)")
        }
        if ($Content -match $secretRx) {
            $found.Add("KEY MATTER $RelPath  carries the secret-encrypted key-envelope marker (key material never enters the repo)")
        }
    }
    return $found
}

# The ONE scan loop, shared by every mode including the self-test
# (round-1 F12: the self-test exercises the real path, not a shortcut).
# $Files: relative paths. $ReadContent: scriptblock(relPath) -> hashtable
# @{ok=<string>} or @{fail=<reason>}. A read failure is a VIOLATION - the
# scanner FAILS CLOSED (round-2 F12): unreadable content is never treated
# as clean.
function Invoke-ScanFiles {
    param([string[]]$Files, [scriptblock]$ReadContent, [string[]]$Allowed)
    $all = [System.Collections.Generic.List[string]]::new()
    foreach ($f in $Files) {
        if (-not $f) { continue }
        $content = $null
        $ext = [System.IO.Path]::GetExtension($f).ToLowerInvariant()
        if ($skipContentExt -notcontains $ext) {
            $result = & $ReadContent $f
            if ($result.ContainsKey('fail')) {
                $all.Add("READ FAIL  $f  cannot be read for scanning ($($result.fail)); fix or remove it - unreadable is never clean (I3)")
                continue
            }
            $content = $result.ok
        }
        foreach ($v in (Test-FileViolations -RelPath $f -Content $content -Allowed $Allowed)) {
            $all.Add($v)
        }
    }
    return $all
}

if ($SelfTest) {
    # Positive fixtures are GENERATED here at runtime, written to disk,
    # scanned through the REAL loop with the working-tree reader, and
    # removed after: nothing marker-shaped is ever committed, so the
    # tripwire is proven live without tripping itself.
    $tempRoot = Join-Path ([System.IO.Path]::GetTempPath()) "cn-pii-selftest-$([guid]::NewGuid())"
    New-Item -ItemType Directory -Force $tempRoot | Out-Null
    try {
        $cases = @(
            @{ Name = 'fake.record.json';    Content = '{}';                                       Expect = 'QUEUE PATH' },
            @{ Name = 'fake.sidecar.json';   Content = '{}';                                       Expect = 'QUEUE PATH' },
            @{ Name = 'fake.reviewed';       Content = 'x';                                        Expect = 'QUEUE PATH' },
            @{ Name = 'payload.json';        Content = '{ "queue_record_version": "9.9.9" }';      Expect = 'QUEUE DATA' },
            @{ Name = 'renamed-intake.md';   Content = '{ "queue_record_version": "9.9.9" }';      Expect = 'QUEUE DATA' },
            @{ Name = 'envelope.txt';        Content = 'age header: secret-encrypted key follows'; Expect = 'KEY MATTER' },
            @{ Name = 'recovery-note.md';    Content = 'the secret-encrypted usb copy';            Expect = 'KEY MATTER' },
            @{ Name = 'contact.txt';         Content = 'reach me at realperson@somewhere.org';     Expect = 'EMAIL' }
        )
        foreach ($case in $cases) {
            Set-Content -Path (Join-Path $tempRoot $case.Name) -Value $case.Content -NoNewline
        }
        # Exemption negative: a QUOTED marker in a .rs source is exempt by
        # extension - this string WOULD match the regex otherwise.
        $exemptName = 'record.rs'
        Set-Content -Path (Join-Path $tempRoot $exemptName) -Value 'let key = "queue_record_version": ;' -NoNewline

        $reader = {
            param($rel)
            $full = Join-Path $tempRoot $rel
            try { return @{ ok = (Get-Content -Raw -ErrorAction Stop $full) } }
            catch { return @{ fail = "$_" } }
        }
        # A missing file exercises the fail-closed READ FAIL rule.
        $names = @($cases | ForEach-Object { $_.Name }) + $exemptName + 'missing-file.txt'
        $hits = Invoke-ScanFiles -Files $names -ReadContent $reader -Allowed @()

        $failures = 0
        foreach ($case in $cases) {
            $matched = @($hits | Where-Object { $_.StartsWith($case.Expect) -and $_.Contains($case.Name) })
            if ($matched.Count -eq 0) {
                Write-Host "SELF-TEST FAIL: '$($case.Name)' did not trip the $($case.Expect) rule" -ForegroundColor Red
                $failures++
            }
        }
        $exemptHits = @($hits | Where-Object { $_.Contains($exemptName) })
        if ($exemptHits.Count -ne 0) {
            Write-Host "SELF-TEST FAIL: exempt source '$exemptName' tripped: $exemptHits" -ForegroundColor Red
            $failures++
        }
        $readFailHits = @($hits | Where-Object { $_.StartsWith('READ FAIL') -and $_.Contains('missing-file.txt') })
        if ($readFailHits.Count -eq 0) {
            Write-Host "SELF-TEST FAIL: unreadable file did not produce READ FAIL (fail-closed rule)" -ForegroundColor Red
            $failures++
        }
        if ($failures -gt 0) { exit 1 }
        Write-Host "PII self-test passed: $($cases.Count) on-disk fixture(s) flagged through the real scan loop, exemption negative holds, fixtures removed."
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
    $reader = {
        param($rel)
        $content = (& git -C $repoRoot show ":$rel" 2>$null) -join "`n"
        if ($LASTEXITCODE -ne 0) { return @{ fail = "git show exited $LASTEXITCODE" } }
        return @{ ok = $content }
    }
} else {
    $tracked   = @(& git -C $repoRoot ls-files)
    $untracked = @(& git -C $repoRoot ls-files --others --exclude-standard)
    $files = @($tracked + $untracked | Sort-Object -Unique)
    $reader = {
        param($rel)
        $full = Join-Path $repoRoot $rel
        if (-not (Test-Path $full -PathType Leaf)) {
            return @{ fail = 'listed by git but not a readable file on disk' }
        }
        try { return @{ ok = (Get-Content -Raw -ErrorAction Stop $full) } }
        catch { return @{ fail = "$_" } }
    }
}

$violations = Invoke-ScanFiles -Files $files -ReadContent $reader -Allowed $allowed

if ($violations.Count -gt 0) {
    Write-Host "PII SCAN FAILED - $($violations.Count) violation(s):" -ForegroundColor Red
    $violations | Sort-Object -Unique | ForEach-Object { Write-Host "  $_" -ForegroundColor Red }
    Write-Host "Fix the content, or for a confirmed false positive add a commented entry to $allowlistFile."
    exit 1
}

Write-Host "PII scan clean ($($files.Count) file(s) checked$(if ($Staged) { ', staged mode' }))."
exit 0
