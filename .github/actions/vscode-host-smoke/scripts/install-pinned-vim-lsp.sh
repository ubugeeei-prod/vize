#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/actions/vscode-host-smoke/scripts/install-pinned-vim-lsp.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
git init "${RUNNER_TEMP}/vim-lsp"
git -C "${RUNNER_TEMP}/vim-lsp" remote add origin https://github.com/prabirshrestha/vim-lsp.git
git -C "${RUNNER_TEMP}/vim-lsp" fetch --depth 1 origin "${VIM_LSP_COMMIT}"
git -C "${RUNNER_TEMP}/vim-lsp" checkout --detach FETCH_HEAD
cd "${RUNNER_TEMP}/vim-lsp"
test "$(git rev-parse HEAD)" = "${VIM_LSP_COMMIT}"
