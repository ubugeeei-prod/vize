#!/usr/bin/env bash
set -euo pipefail
if [ "${CLIENT}" = "vscode" ]; then
  xvfb-run -a node tests/editor-conformance/run.ts "${CLIENT}" --out "${RUNNER_TEMP}/ts45"
else
  node tests/editor-conformance/run.ts "${CLIENT}" --out "${RUNNER_TEMP}/ts45"
fi
