#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/typechecker-native-phases/pin-immutable-driver-source-and-freshly-fetched-main-baseline.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
[[ "$DRIVER_SHA" =~ ^[0-9a-f]{40}$ && "$SOURCE_SHA" =~ ^[0-9a-f]{40}$ ]]
[[ "$(git rev-parse HEAD)" == "$DRIVER_SHA" ]]
git fetch --no-tags origin +refs/heads/main:refs/remotes/origin/main
main_head_sha="$(git rev-parse refs/remotes/origin/main)"
[[ "$(git rev-parse --verify "$SOURCE_SHA^{commit}")" == "$SOURCE_SHA" ]]
[[ "$GITHUB_REPOSITORY" == ubugeeei-prod/vize && "$GITHUB_REF" == refs/heads/main ]]
[[ "$DRIVER_SHA" == "$GITHUB_WORKFLOW_SHA" ]]
git merge-base --is-ancestor "$DRIVER_SHA" "$main_head_sha"
git merge-base --is-ancestor "$SOURCE_SHA" "$main_head_sha"
if [[ "$GITHUB_EVENT_NAME" == schedule ]]; then
  [[ "$SOURCE_SHA" == "$DRIVER_SHA" && "$SOURCE_SHA" == "$GITHUB_SHA" ]]
else
  [[ "$GITHUB_EVENT_NAME" == workflow_dispatch ]]
fi
if [[ "$HISTORICAL_PROJECTION_RECEIPT" == true ]]; then
  [[ "$GITHUB_EVENT_NAME" == workflow_dispatch ]]
  [[ "$SOURCE_SHA" == "$DRIVER_SHA" && "$SOURCE_SHA" == "$GITHUB_SHA" ]]
fi
source_base_sha="$SOURCE_SHA"
echo "MAIN_SOURCE_SHA=$source_base_sha" >> "$GITHUB_ENV"
echo "MAIN_HEAD_SHA=$main_head_sha" >> "$GITHUB_ENV"
echo "NATIVE_PHASE_SOURCE_ROOT=$GITHUB_WORKSPACE" >> "$GITHUB_ENV"
echo "NATIVE_PHASE_OUTPUT=$RUNNER_TEMP/typechecker-native-phases.json" >> "$GITHUB_ENV"
