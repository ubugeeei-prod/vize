#!/usr/bin/env bash
set -euo pipefail
evidence="$RUNNER_TEMP/original27-current-evidence"
export CARGO_TARGET_DIR="$RUNNER_TEMP/original27-current-target"
(cd original27-source && cargo test --locked --profile ci-opt -p vize_canon --lib --no-run --message-format=json) > "$evidence/origin-cargo.jsonl" 2> "$evidence/origin-cargo.stderr"
sidecar_args=()
if [[ "$NATIVE_VUE_RETURN_OBSERVATION" == true ]]; then
  sidecar_args+=(--vue-return-observation)
fi
node tools/support/compat/content-mapper/constructor-native-props-origin.ts build "$GITHUB_WORKSPACE/original27-source" "$CAPTURE_SOURCE" "$GITHUB_WORKSPACE/original27-fixtures" "$RUNNER_TEMP/original27-current-provider" "$evidence" "${sidecar_args[@]}"
