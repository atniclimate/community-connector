#Requires -Version 7
<#
.SYNOPSIS
  Reveal launcher (S-R7): one command to stand up the app and open it, for the
  ATNI convention demo.

.DESCRIPTION
  Rebuilds the cn-wasm package if core/crates/cn-wasm/pkg is missing, starts
  the app (dev server, or a built preview with -Built), waits until the port
  answers, then opens the default browser at the app URL for -Group
  (default atni-convention, the synthetic convention fixture).

  The app reads one query parameter, `group` (app/src/main.ts DEV_GROUPS):
  research-network or atni-convention. Presenter mode has no parameter; it is
  entered with the in-app "Present" button.

  -Built serves the production build, which has no fixture loader: the graph
  is empty. Use the default dev server for the reveal.

.EXAMPLE
  pwsh scripts/reveal.ps1
  pwsh scripts/reveal.ps1 -Group research-network
#>
[CmdletBinding()]
param(
    [ValidateSet('atni-convention', 'research-network')]
    [string]$Group = 'atni-convention',
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
$url = "http://localhost:$Port/?group=$Group"

$npxCmd = (Get-Command npx.cmd, npx -ErrorAction SilentlyContinue | Select-Object -First 1).Source
if (-not $npxCmd) {
    Write-Host 'reveal: npx not found on PATH.' -ForegroundColor Red
    exit 2
}

if ($Built) {
    Write-Host 'reveal: -Built has no fixture loader, so the graph will be empty. Omit -Built for the reveal.' -ForegroundColor Yellow
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
