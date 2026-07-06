# Installs repo git hooks. Run once per clone (AGENTS.md I1 tripwire).
$ErrorActionPreference = 'Stop'
& git config core.hooksPath scripts/hooks
Write-Host "core.hooksPath -> scripts/hooks (pre-commit PII tripwire active)."
