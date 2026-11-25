#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/03-vscode-startup/scripts/stage-the-published-server-and-vs-code.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
[[ "$ROUNDS" =~ ^[1-9][0-9]?$ ]] && [[ "$SFCS" =~ ^[0-9]+$ ]] && [[ "$NOISE" =~ ^[0-9]+$ ]]
mkdir -p "$RUNNER_TEMP/server"
cp ext-builds/server/vize "$RUNNER_TEMP/server/vize" && chmod +x "$RUNNER_TEMP/server/vize"
"$RUNNER_TEMP/server/vize" --version
if ! command -v xvfb-run >/dev/null 2>&1; then
  sudo apt-get update -qq && sudo apt-get install -y --no-install-recommends xvfb xauth
fi
VIZE_TEST_VSCODE_VERSION="$VSCODE_VERSION" vp run --workspace-root test:vscode-extension:download
code=$(cd editors/vscode && VIZE_TEST_VSCODE_VERSION="$VSCODE_VERSION" node test/download-vscode.mjs | sed -n -f "$VIZE_CI_FILES/verified-editor-path.sed" | tail -1)
test -x "$code"
echo "CODE_EXE=$code" >> "$GITHUB_ENV"
