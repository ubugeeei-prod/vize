#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/editor-conformance/prepare-pinned-vs-code.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
if ! command -v xvfb-run >/dev/null 2>&1; then
  vize-ci-apt-retry sudo apt-get update
  vize-ci-apt-retry sudo apt-get install -y xvfb
fi
timeout --signal=TERM --kill-after=10s 300s vp run --workspace-root test:vscode-extension:download
