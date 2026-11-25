#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/release-zed-host/install-the-pinned-zed-extension-packager.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
timeout --signal=TERM --kill-after=10s 300s \
  bash .github/actions/vscode-host-smoke/download-artifact.sh \
  "https://zed-extension-cli.nyc3.digitaloceanspaces.com/${ZED_EXTENSION_CLI_SHA}/x86_64-unknown-linux-gnu/zed-extension" \
  "${RUNNER_TEMP}/zed-extension" "${ZED_EXTENSION_CLI_SHA256}"
chmod +x "${RUNNER_TEMP}/zed-extension"
