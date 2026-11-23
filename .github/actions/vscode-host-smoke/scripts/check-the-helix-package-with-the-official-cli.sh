#!/usr/bin/env bash
set -euo pipefail
mkdir -p "${XDG_CONFIG_HOME}/helix"
cp editors/helix/languages.toml "${XDG_CONFIG_HOME}/helix/languages.toml"
node tools/commands/editors/helix/assert-helix-health.ts "${RUNNER_TEMP}/helix/hx"
