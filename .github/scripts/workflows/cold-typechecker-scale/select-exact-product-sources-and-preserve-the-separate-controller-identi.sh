#!/usr/bin/env bash
set -euo pipefail
[[ "${VIZE_INPUTS_SFCS}" == 2500 ]]
mkdir "$RUNNER_TEMP/cold-paired"
selection=$(node tools/benchmarks/scripts/cold-typechecker-scale-candidate-plan.ts --plan tools/benchmarks/scripts/cold-typechecker-scale-candidate-cases.json --out "$RUNNER_TEMP/cold-paired/selection" --base "$BASE_SHA" --head "$HEAD_SHA" --sfcs "${VIZE_INPUTS_SFCS}")
[[ "$selection" == PAIRED_ROUTE=legacy || "$selection" == PAIRED_ROUTE=candidate ]]
printf '%s\n' "$selection" >> "$GITHUB_ENV"
if [[ "$selection" == PAIRED_ROUTE=legacy ]]; then
  git fetch --no-tags --depth=1 "https://github.com/$GITHUB_REPOSITORY.git" "$BASE_SHA" "$HEAD_SHA"
elif [[ "$(git rev-parse --is-shallow-repository)" == true ]]; then
  git fetch --no-tags --unshallow "https://github.com/$GITHUB_REPOSITORY.git" "$BASE_SHA" "$HEAD_SHA"
else
  git fetch --no-tags "https://github.com/$GITHUB_REPOSITORY.git" "$BASE_SHA" "$HEAD_SHA"
fi
git worktree add --detach "$RUNNER_TEMP/cold-paired-base" "$BASE_SHA"
git worktree add --detach "$RUNNER_TEMP/cold-paired-head" "$HEAD_SHA"
git rev-parse HEAD HEAD^{tree} > "$RUNNER_TEMP/cold-paired/driver.txt"
