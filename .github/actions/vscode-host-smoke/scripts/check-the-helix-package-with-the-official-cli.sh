#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/actions/vscode-host-smoke/scripts/check-the-helix-package-with-the-official-cli.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
mkdir -p "${XDG_CONFIG_HOME}/helix"
cp editors/helix/languages.toml "${XDG_CONFIG_HOME}/helix/languages.toml"
node tools/commands/editors/helix/assert-helix-health.ts "${RUNNER_TEMP}/helix/hx"
