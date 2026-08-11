#!/usr/bin/env bash
# Internal bash entry point for the remote-intake form-to-graph rehearsal (relay
# blueprint step 9). Thin delegate to the PowerShell orchestrator so the real
# logic lives in exactly one place (scripts/e2e-remote-intake.ps1). Pass
# PowerShell-style flags through, e.g.:  scripts/e2e-remote-intake.sh -Port 8791
set -euo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
exec pwsh "${here}/e2e-remote-intake.ps1" "$@"
