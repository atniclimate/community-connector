#!/usr/bin/env bash
# Internal bash entry point for the remote intake form build (relay blueprint
# step 6). Thin delegate to the PowerShell orchestrator so the real logic lives
# in exactly one place (scripts/build-form.ps1). Pass PowerShell-style flags
# through, e.g.:  scripts/build-form.sh -CheckReproducible
set -euo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
exec pwsh "${here}/build-form.ps1" "$@"
