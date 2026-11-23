#!/usr/bin/env bash
set -euo pipefail
sidecar_args=()
if [[ "$NATIVE_VUE_RETURN_OBSERVATION" == true ]]; then
  sidecar_args+=(--vue-return-observation)
fi
node tools/support/compat/content-mapper/constructor-native-props-origin.ts run "$GITHUB_WORKSPACE/original27-source" "$CAPTURE_SOURCE" "$GITHUB_WORKSPACE/original27-fixtures" "$RUNNER_TEMP/original27-current-provider" "$RUNNER_TEMP/original27-current-evidence" "${sidecar_args[@]}"
