#Requires -Version 7
<#
check-all - single verification orchestrator (PLAN_1.0.md P0.2; AGENTS.md
"Repo commands" AUTOMATED list). Runs the full battery in a fixed order,
runs every selected member even after a failure, prints a summary table,
and exits non-zero if ANY member failed. Failures are collected and
surfaced, never swallowed (AGENTS.md I3).

Members (in order):
  rust-fmt        core/  cargo fmt --check
  rust-clippy     core/  cargo clippy --workspace --all-targets -- -D warnings
  rust-test       core/  cargo test --workspace
  wasm-build      core/  wasm-pack build crates/cn-wasm --target web
  app-typecheck   app/   npm run typecheck
  app-build       app/   npm run build
  app-test        app/   npm run test
  app-templates   app/   npm run validate:templates
  app-smoke       app/   npm run smoke:node
  app-snapshot    app/   npm run build:snapshot   (includes the 5MB size gate, I8)
  pii-scan        root   pwsh scripts/pii-scan.ps1  (always runs, every mode)

Usage:
  pwsh scripts/check-all.ps1                # full battery
  pwsh scripts/check-all.ps1 -RustOnly      # rust members + PII scan
  pwsh scripts/check-all.ps1 -AppOnly       # app members + PII scan
  pwsh scripts/check-all.ps1 -SkipSnapshot  # omit app-snapshot (composes with any mode)
  pwsh scripts/check-all.ps1 -Staged        # pre-commit mode: members are selected by
                                            # which top-level paths (core/, app/,
                                            # schemas/, fixtures/, scripts/) appear in
                                            # the staged diff; the PII scan runs in
                                            # staged mode and always runs
  -Quiet                                    # buffer member output; print it only for
                                            # failing members (hook mode stays terse)

Staged trigger map (kept proportional so atomic commits stay cheap):
  core/      -> rust-fmt, rust-clippy, rust-test, wasm-build, app-smoke, app-snapshot
  app/       -> app-typecheck, app-build, app-test, app-templates, app-smoke, app-snapshot
  schemas/   -> app-typecheck, app-test, app-templates
  fixtures/  -> app-test, app-templates, app-smoke
  scripts/   -> app-snapshot (exercises check-size.mjs)
  (any/none) -> pii-scan always
#>
param(
    [switch]$RustOnly,
    [switch]$AppOnly,
    [switch]$SkipSnapshot,
    [switch]$Staged,
    [switch]$Quiet
)

$ErrorActionPreference = 'Stop'
$repoRoot = (& git rev-parse --show-toplevel).Trim()
if ($LASTEXITCODE -ne 0 -or -not $repoRoot) {
    Write-Host 'check-all: not inside a git repository.' -ForegroundColor Red
    exit 2
}

$modeCount = @($RustOnly, $AppOnly, $Staged).Where({ $_ }).Count
if ($modeCount -gt 1) {
    Write-Host 'check-all: -RustOnly, -AppOnly, and -Staged are mutually exclusive.' -ForegroundColor Red
    exit 2
}

# Member table. Group: rust | app | cross. StagedTriggers: top-level repo paths
# whose presence in the staged diff selects the member in -Staged mode.
$members = @(
    @{ Name = 'rust-fmt';      Group = 'rust';  Dir = 'core'; Exe = 'cargo';
       Args = @('fmt', '--check');
       StagedTriggers = @('core') }
    @{ Name = 'rust-clippy';   Group = 'rust';  Dir = 'core'; Exe = 'cargo';
       Args = @('clippy', '--workspace', '--all-targets', '--', '-D', 'warnings');
       StagedTriggers = @('core') }
    @{ Name = 'rust-test';     Group = 'rust';  Dir = 'core'; Exe = 'cargo';
       Args = @('test', '--workspace');
       StagedTriggers = @('core') }
    @{ Name = 'wasm-build';    Group = 'rust';  Dir = 'core'; Exe = 'wasm-pack';
       Args = @('build', 'crates/cn-wasm', '--target', 'web');
       StagedTriggers = @('core') }
    @{ Name = 'app-typecheck'; Group = 'app';   Dir = 'app';  Exe = 'npm';
       Args = @('run', 'typecheck');
       StagedTriggers = @('app', 'schemas') }
    @{ Name = 'app-build';     Group = 'app';   Dir = 'app';  Exe = 'npm';
       Args = @('run', 'build');
       StagedTriggers = @('app') }
    @{ Name = 'app-test';      Group = 'app';   Dir = 'app';  Exe = 'npm';
       Args = @('run', 'test');
       StagedTriggers = @('app', 'schemas', 'fixtures') }
    @{ Name = 'app-templates'; Group = 'app';   Dir = 'app';  Exe = 'npm';
       Args = @('run', 'validate:templates');
       StagedTriggers = @('app', 'schemas', 'fixtures') }
    @{ Name = 'app-smoke';     Group = 'app';   Dir = 'app';  Exe = 'npm';
       Args = @('run', 'smoke:node');
       StagedTriggers = @('app', 'fixtures', 'core') }
    @{ Name = 'app-snapshot';  Group = 'app';   Dir = 'app';  Exe = 'npm';
       Args = @('run', 'build:snapshot');
       StagedTriggers = @('app', 'core', 'scripts') }
    @{ Name = 'pii-scan';      Group = 'cross'; Dir = '.';    Exe = 'pwsh';
       Args = @('-NoProfile', '-File', 'scripts/pii-scan.ps1');
       StagedTriggers = @() }  # selected unconditionally below
)

# --- Selection ---------------------------------------------------------------
$selected = [System.Collections.Generic.List[hashtable]]::new()

if ($Staged) {
    $stagedFiles = @(& git -C $repoRoot diff --cached --name-only --diff-filter=ACMRD)
    if ($LASTEXITCODE -ne 0) {
        Write-Host 'check-all: git diff --cached failed.' -ForegroundColor Red
        exit 2
    }
    $stagedTops = @($stagedFiles | ForEach-Object { ($_ -split '/')[0] } | Sort-Object -Unique)
    foreach ($m in $members) {
        if ($m.Name -eq 'pii-scan') { $selected.Add($m); continue }
        foreach ($t in $m.StagedTriggers) {
            if ($stagedTops -contains $t) { $selected.Add($m); break }
        }
    }
} elseif ($RustOnly) {
    foreach ($m in $members) { if ($m.Group -in @('rust', 'cross')) { $selected.Add($m) } }
} elseif ($AppOnly) {
    foreach ($m in $members) { if ($m.Group -in @('app', 'cross')) { $selected.Add($m) } }
} else {
    foreach ($m in $members) { $selected.Add($m) }
}

if ($SkipSnapshot) {
    $selected = [System.Collections.Generic.List[hashtable]]@(
        $selected | Where-Object { $_.Name -ne 'app-snapshot' })
}

# The PII scan runs staged-scoped in -Staged mode (pre-commit semantics).
if ($Staged) {
    $pii = $selected | Where-Object { $_.Name -eq 'pii-scan' }
    $pii.Args = @($pii.Args) + '-Staged'
}

if ($selected.Count -eq 0) {
    Write-Host 'check-all: no members selected.' -ForegroundColor Yellow
    exit 0
}

# --- Execution ---------------------------------------------------------------
$results = [System.Collections.Generic.List[pscustomobject]]::new()

foreach ($m in $selected) {
    $dir = Join-Path $repoRoot $m.Dir
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    $exit = 0
    $buffered = $null
    Push-Location $dir
    try {
        if ($Quiet) {
            $buffered = & $m.Exe @($m.Args) 2>&1
        } else {
            Write-Host "== $($m.Name): $($m.Exe) $($m.Args -join ' ')  (in $($m.Dir))" -ForegroundColor Cyan
            & $m.Exe @($m.Args) 2>&1 | ForEach-Object { $_ }
        }
        $exit = $LASTEXITCODE
    } catch {
        # A member whose executable is missing or throws is a FAILURE, never
        # skipped (I3). Record the error text and keep going.
        $exit = 127
        $buffered = @($buffered) + $_.Exception.Message
        if (-not $Quiet) { Write-Host $_.Exception.Message -ForegroundColor Red }
    } finally {
        Pop-Location
    }
    $sw.Stop()
    $status = if ($exit -eq 0) { 'PASS' } else { 'FAIL' }
    if ($Quiet) {
        if ($exit -ne 0) {
            Write-Host "FAIL $($m.Name) (exit $exit) - output:" -ForegroundColor Red
            $buffered | ForEach-Object { Write-Host "  $_" }
        } else {
            Write-Host "PASS $($m.Name) ($([math]::Round($sw.Elapsed.TotalSeconds, 1))s)"
        }
    }
    $results.Add([pscustomobject]@{
        Member  = $m.Name
        Status  = $status
        Seconds = [math]::Round($sw.Elapsed.TotalSeconds, 1)
        Exit    = $exit
    })
}

# --- Summary -----------------------------------------------------------------
$failed = @($results | Where-Object { $_.Status -eq 'FAIL' })
Write-Host ''
Write-Host 'check-all summary:'
$results | Format-Table -AutoSize Member, Status, Seconds, Exit | Out-String |
    ForEach-Object { Write-Host $_.TrimEnd() }

if ($failed.Count -gt 0) {
    Write-Host "check-all: $($failed.Count) of $($results.Count) member(s) FAILED: $(($failed.Member) -join ', ')" -ForegroundColor Red
    exit 1
}
Write-Host "check-all: all $($results.Count) member(s) passed." -ForegroundColor Green
exit 0
