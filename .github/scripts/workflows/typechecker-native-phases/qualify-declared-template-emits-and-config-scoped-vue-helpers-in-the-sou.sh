#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/typechecker-native-phases/qualify-declared-template-emits-and-config-scoped-vue-helpers-in-the-sou.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
[[ "$GITHUB_WORKFLOW_SHA" =~ ^[0-9a-f]{40}$ ]] && git fetch --no-tags origin "$GITHUB_WORKFLOW_SHA"
for helper in typechecker-native-source-recipe.mjs typechecker-native-release-metadata.mjs; do git show "$GITHUB_WORKFLOW_SHA:tools/benchmarks/scripts/$helper" > "$RUNNER_TEMP/$helper"; done
if [[ "$GITHUB_EVENT_NAME" == workflow_dispatch ]]; then
  [[ "$HISTORICAL_PROJECTION_RECEIPT" == true && "$SOURCE_SHA" == "$DRIVER_SHA" ]]
  recipe=""
else
  recipe="$(node "$RUNNER_TEMP/typechecker-native-source-recipe.mjs")"
fi
if [[ -n "$recipe" ]]; then
  if [[ "$(node "$VIZE_CI_FILES/has-tsconfig-capture.cjs")" != true ]]; then
    unset VIZE_TSCONFIG_TYPES_CAPTURE
  fi
  unset GH_TOKEN
  export VIZE_NATIVE_RECIPE_FILES="$RUNNER_TEMP/native-source-files"
  bash --noprofile --norc -e -o pipefail "$recipe"
else
  export VIZE_NATIVE_RECIPE_FILES="$VIZE_CI_FILES"
  bash --noprofile --norc -e -o pipefail "$VIZE_CI_FILES/run-native-qualification.sh"
fi
