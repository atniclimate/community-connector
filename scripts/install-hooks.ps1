# Installs repo git hooks. Run once per clone (AGENTS.md I1 tripwire + P0.3 gate).
$ErrorActionPreference = 'Stop'
& git config core.hooksPath scripts/hooks
if ($LASTEXITCODE -ne 0) {
    Write-Host 'install-hooks: git config core.hooksPath failed - hooks are NOT installed.' -ForegroundColor Red
    exit 1
}
Write-Host "core.hooksPath -> scripts/hooks (pre-commit: staged PII scan always; scoped check-all -Staged when code paths are staged)."
