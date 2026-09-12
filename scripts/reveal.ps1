#Requires -Version 7
<#
.SYNOPSIS
  Reveal launcher (S-R7): one command to stand up the app and open it, for the
  ATNI convention demo.

.DESCRIPTION
  Rebuilds the cn-wasm package if core/crates/cn-wasm/pkg is missing, starts
  the app (dev server, or a built preview with -Built), waits until the port
  answers, then opens the default browser at the app URL.

  Note on query parameters: app/src/main.ts (checked 2026-09-12) reads no URL
  query parameters at all - there is no `mode` or `group` parameter to enter
  presenter mode or pick a group. Presenter mode is entered in-app via the
  "Present" toolbar button once beats have loaded. The dev server always
  loads the research-network demo fixture (main.ts's hardcoded DEMO_GROUP_ID);
  switching that to atni-convention is an app-code change and out of scope
  for this script (see SESSION_ROSTER.yaml S-R7). So this launcher opens the
  plain app URL with no query string.

.EXAMPLE
  pwsh scripts/reveal.ps1
  pwsh scripts/reveal.ps1 -Built
#>
[CmdletBinding()]
param(
    [switch]$Built,
    [int]$Port,
    [int]$TimeoutSeconds = 60
)

$ErrorActionPreference = 'Stop'
$repoRoot = (& git rev-parse --show-toplevel).Trim()
if ($LASTEXITCODE -ne 0 -or -not $repoRoot) {
    Write-Host 'reveal: not inside a git repository.' -ForegroundColor Red
    exit 2
}
$appDir = Join-Path $repoRoot 'app'
$wasmPkgDir = Join-Path $repoRoot 'core/crates/cn-wasm/pkg'

if (-not (Test-Path $wasmPkgDir)) {
    Write-Host "==> cn-wasm pkg missing, building it (wasm-pack build crates/cn-wasm --target web) ..."
    if (-not (Get-Command wasm-pack -ErrorAction SilentlyContinue)) {
        Write-Host 'reveal: wasm-pack not found on PATH; install it or build core/crates/cn-wasm/pkg manually.' -ForegroundColor Red
        exit 2
    }
    Push-Location (Join-Path $repoRoot 'core')
    try {
        & wasm-pack build crates/cn-wasm --target web
        if ($LASTEXITCODE -ne 0) { throw 'wasm-pack build failed' }
    } finally {
        Pop-Location
    }
}

if ($Port -eq 0) { $Port = if ($Built) { 4173 } else { 5173 } }
$url = "http://localhost:$Port/"

$npxCmd = (Get-Command npx.cmd, npx -ErrorAction SilentlyContinue | Select-Object -First 1).Source
if (-not $npxCmd) {
    Write-Host 'reveal: npx not found on PATH.' -ForegroundColor Red
    exit 2
}

if ($Built) {
    Write-Host '==> npm run build (app/) ...'
    & npm --prefix $appDir run build
    if ($LASTEXITCODE -ne 0) { throw 'npm run build failed' }
    Write-Host "==> starting built preview on port $Port ..."
    $proc = Start-Process -FilePath $npxCmd `
        -ArgumentList @('vite', 'preview', '--port', "$Port", '--strictPort') `
        -WorkingDirectory $appDir -PassThru -WindowStyle Hidden
} else {
    Write-Host "==> starting dev server on port $Port ..."
    $proc = Start-Process -FilePath $npxCmd `
        -ArgumentList @('vite', '--port', "$Port", '--strictPort') `
        -WorkingDirectory $appDir -PassThru -WindowStyle Hidden
}

$deadline = (Get-Date).AddSeconds($TimeoutSeconds)
$up = $false
while ((Get-Date) -lt $deadline) {
    if ($proc.HasExited) {
        Write-Host "reveal: server process exited early (code $($proc.ExitCode)); port $Port may already be in use." -ForegroundColor Red
        exit 1
    }
    try {
        $client = [System.Net.Sockets.TcpClient]::new()
        $client.Connect('localhost', $Port)
        $client.Close()
        $up = $true
        break
    } catch {
        Start-Sleep -Milliseconds 300
    }
}

if (-not $up) {
    Write-Host "reveal: port $Port did not answer within $TimeoutSeconds seconds." -ForegroundColor Red
    exit 1
}

Start-Process $url
Write-Host $url
Write-Host 'Escape exits presenter mode.'
