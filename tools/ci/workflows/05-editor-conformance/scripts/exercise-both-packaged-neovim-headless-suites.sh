#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/05-editor-conformance/scripts/exercise-both-packaged-neovim-headless-suites.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
PATH="$(dirname "${VIZE_TEST_NVIM_PATH}"):$PATH"
export PATH
test "$(command -v nvim)" = "${VIZE_TEST_NVIM_PATH}"
command -v nvim
nvim --version | head -1
vp run --workspace-root test:nvim-extension:headless
