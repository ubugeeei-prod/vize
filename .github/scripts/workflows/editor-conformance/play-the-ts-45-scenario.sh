#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/editor-conformance/play-the-ts-45-scenario.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
if [ "${CLIENT}" = "vscode" ]; then
  xvfb-run -a node tests/editor-conformance/run.ts "${CLIENT}" --out "${RUNNER_TEMP}/ts45"
else
  node tests/editor-conformance/run.ts "${CLIENT}" --out "${RUNNER_TEMP}/ts45"
fi
