#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/editor-conformance/exercise-project-configuration-through-the-actual-client.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
if [ "$CLIENT" = vim ]; then
  node tools/support/compat/vim-vize/run-config-server.ts
else
  node tools/support/compat/emacs-vize/run-real-server.ts
fi
node tests/editor-conformance/server-artifact.ts host "${RUNNER_TEMP}/vize-server/server.json" "${RUNNER_TEMP}/configuration-client/receipt.json" "$GITHUB_SHA"
