#!/usr/bin/env bash
if [[ "$RUNNER_OS" == Linux ]]; then
  xvfb-run -a vp node editors/vscode/test/run-typescript-provider-host.mjs
else
  vp node editors/vscode/test/run-typescript-provider-host.mjs
fi
