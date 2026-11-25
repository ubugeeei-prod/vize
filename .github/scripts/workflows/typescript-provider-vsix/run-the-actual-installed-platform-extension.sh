#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/typescript-provider-vsix/run-the-actual-installed-platform-extension.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
if [[ "$RUNNER_OS" == Linux ]]; then
  xvfb-run -a vp node editors/vscode/test/run-typescript-provider-host.mjs
else
  vp node editors/vscode/test/run-typescript-provider-host.mjs
fi
