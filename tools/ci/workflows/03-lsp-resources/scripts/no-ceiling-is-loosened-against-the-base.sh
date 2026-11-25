#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/03-lsp-resources/scripts/no-ceiling-is-loosened-against-the-base.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
git fetch --no-tags --depth=1 origin "${BASE_SHA}"
# Resolve the base revision's path so the directory move itself can
# still compare against the pre-move budget registry.
BASE_BUDGET_PATH="$(git ls-tree -r --name-only "${BASE_SHA}" | rg '(^|/)plan/budgets[.]toml$')"
test -n "${BASE_BUDGET_PATH}"
git show "${BASE_SHA}:${BASE_BUDGET_PATH}" > "${RUNNER_TEMP}/base-budgets.toml"
rust-script tools/commands/davinci/resource-budgets.rs --ratchet "${RUNNER_TEMP}/base-budgets.toml"
