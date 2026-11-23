#!/usr/bin/env bash
set -euo pipefail
git init "${RUNNER_TEMP}/vim-lsp"
git -C "${RUNNER_TEMP}/vim-lsp" remote add origin https://github.com/prabirshrestha/vim-lsp.git
git -C "${RUNNER_TEMP}/vim-lsp" fetch --depth 1 origin "$VIM_LSP_COMMIT"
git -C "${RUNNER_TEMP}/vim-lsp" checkout --detach FETCH_HEAD
test "$(git -C "${RUNNER_TEMP}/vim-lsp" rev-parse HEAD)" = "$VIM_LSP_COMMIT"
