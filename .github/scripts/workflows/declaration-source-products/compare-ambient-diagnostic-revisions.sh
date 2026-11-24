#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/declaration-source-products/compare-ambient-diagnostic-revisions.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
[[ "$BASE_SHA" =~ ^[0-9a-f]{40}$ && "$HEAD_SHA" =~ ^[0-9a-f]{40}$ ]]
git fetch --no-tags --depth=1 origin "$BASE_SHA"
work="$RUNNER_TEMP/ambient-read-checkout"
suite=crates/vize_maestro/src/ide/diagnostics/corsa/ambient_project
export CARGO_TARGET_DIR="$GITHUB_WORKSPACE/target"
for side in base head; do
  revision="$BASE_SHA"
  if [[ "$side" == head ]]; then
    git worktree remove --force "$work"
    revision="$HEAD_SHA"
  fi
  git worktree add --quiet --detach "$work" "$revision"
  for dir in node_modules playground/node_modules; do
    if [[ -d "$GITHUB_WORKSPACE/$dir" ]]; then ln -s "$GITHUB_WORKSPACE/$dir" "$work/$dir"; fi
  done
  # Instrument only the test module on the parent. Both executables
  # embed the same manifest path and run with the same pinned SDK.
  if [[ "$side" == base ]]; then
    cp "$GITHUB_WORKSPACE/$suite/tests/cost.rs" "$work/$suite/tests/cost.rs"
    if ! grep -Fxq 'mod cost;' "$work/$suite/tests.rs"; then
      echo 'mod cost;' >> "$work/$suite/tests.rs"
    fi
  fi
  cmp "$GITHUB_WORKSPACE/$suite/tests/cost.rs" "$work/$suite/tests/cost.rs"
  (cd "$work" && cargo test --locked --profile ci-opt -p vize_maestro --lib --features experimental-ambient-project,micro-profiling --no-run --message-format=json) > "$RUNNER_TEMP/ambient-$side.jsonl"
  mapfile -t exe < <(jq -r -f "$VIZE_CI_FILES/native-maestro-test-executable.jq" "$RUNNER_TEMP/ambient-$side.jsonl")
  [[ "${#exe[@]}" == 1 ]]
  cp "${exe[0]}" "$RUNNER_TEMP/ambient-$side"
done
node tools/benchmarks/scripts/compare-ambient-diagnostics.ts "$RUNNER_TEMP/ambient-base" "$RUNNER_TEMP/ambient-head" "$RUNNER_TEMP/ambient-diagnostics.json" | tee -a "$GITHUB_STEP_SUMMARY"
