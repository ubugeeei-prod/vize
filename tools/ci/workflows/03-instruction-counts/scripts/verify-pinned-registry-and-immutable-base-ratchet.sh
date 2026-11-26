#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/03-instruction-counts/scripts/verify-pinned-registry-and-immutable-base-ratchet.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
if [[ -z "$BASE_SHA" ]]; then
  BASE_SHA=$(git ls-remote origin refs/heads/main | cut -f1)
fi
[[ "$BASE_SHA" =~ ^[a-f0-9]{40}$ ]]
git fetch --no-tags --depth=1 origin "$BASE_SHA"
BASE_ARGS=()
if git cat-file -e "$BASE_SHA:docs/davinci/plan/instruction-budgets.toml" 2>/dev/null; then
  git show "$BASE_SHA:docs/davinci/plan/instruction-budgets.toml" > "${RUNNER_TEMP}/base-instruction-budgets.toml"
  BASE_ARGS=(--base-budgets "${RUNNER_TEMP}/base-instruction-budgets.toml")
else
  # First introduction only: require the exact independently reviewed
  # 100-row artifact. Later bases contain a registry and must ratchet.
  echo '17c3947044c033d03b0a86d13b2c3fa6dc2d1e90cb2222033921f649e97a826f  docs/davinci/plan/instruction-budgets.toml' | sha256sum --check --strict
fi
node tools/benchmarks/scripts/instruction-counts.mjs --verify-budgets "${BASE_ARGS[@]}"
FORMATTER_BASE_ARGS=()
if git cat-file -e "$BASE_SHA:docs/davinci/plan/formatter-instruction-budgets.toml" 2>/dev/null; then
  git show "$BASE_SHA:docs/davinci/plan/formatter-instruction-budgets.toml" > "${RUNNER_TEMP}/base-formatter-instruction-budgets.toml"
  FORMATTER_BASE_ARGS=(--base-budgets "${RUNNER_TEMP}/base-formatter-instruction-budgets.toml")
else
  # First introduction: exact reviewed three-run formatter calibration.
  echo 'e49d7c38a43fcdfa3bf7114c50ef91734f355d943269d42bc1d18821d444de99  docs/davinci/plan/formatter-instruction-budgets.toml' | sha256sum --check --strict
fi
node tools/benchmarks/scripts/instruction-counts.mjs --formatter --verify-budgets "${FORMATTER_BASE_ARGS[@]}"
