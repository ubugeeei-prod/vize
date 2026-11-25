#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/release-zed-host/install-the-pinned-zed-editor.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
archive="${RUNNER_TEMP}/zed-linux-x86_64.tar.gz"
bash .github/actions/vscode-host-smoke/download-artifact.sh \
  "https://github.com/zed-industries/zed/releases/download/v${ZED_VERSION}/zed-linux-x86_64.tar.gz" \
  "$archive" "$ZED_SHA256" \
  "https://api.github.com/repos/zed-industries/zed/releases/assets/710065550"
mkdir -p "${RUNNER_TEMP}/zed"
tar -xzf "$archive" -C "${RUNNER_TEMP}/zed"
"${RUNNER_TEMP}/zed/zed.app/bin/zed" --version
