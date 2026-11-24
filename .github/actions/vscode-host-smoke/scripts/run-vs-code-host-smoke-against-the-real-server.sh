#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/actions/vscode-host-smoke/scripts/run-vs-code-host-smoke-against-the-real-server.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
if [ -n "$PREBUILT_VSIX" ]; then
  xvfb-run -a vp node editors/vscode/test/run-extension-host-real.mjs
else
  xvfb-run -a vp run --workspace-root test:vscode-extension:host-real
fi
