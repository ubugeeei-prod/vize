#!/usr/bin/env bash
set -euo pipefail
evidence="$RUNNER_TEMP/component-binding-evidence"
affinity=$(node "$VIZE_CI_FILES/../shared/runner-resources.ts" first 8)
printf '%s\n' "$affinity" > "$evidence/affinity.txt"
mkdir "$RUNNER_TEMP/component-binding-input"
export VIZE_COMPONENT_PROFILE_FILE="$RUNNER_TEMP/component-binding-input/Bindings.vue"
for round in 1 2 3; do
  order='base head'
  if [[ "$round" == 2 ]]; then order='head base'; fi
  for shape in camel missing dynamic empty; do
    for count in 32 128 512; do
      for role in $order; do
        binary=$(cat "$evidence/$role.profile-binary")
        export VIZE_COMPONENT_PROFILE_SOURCE=$(cat "$evidence/$role.source")
        export VIZE_COMPONENT_PROFILE_SHAPE="$shape" VIZE_COMPONENT_PROFILE_COUNT="$count"
        for mode in runtime-off allocation; do
          name="$role-$shape-$count-$mode-$round"
          export VIZE_COMPONENT_PROFILE_MODE="$mode" VIZE_COMPONENT_PROFILE_OUTPUT="$evidence/$name.json"
          taskset -c "$affinity" "$binary" --exact component_binding_generation_profile --ignored --nocapture > "$evidence/$name.stdout" 2> "$evidence/$name.stderr"
        done
      done
    done
  done
done
taskset -c "$affinity" node tools/benchmarks/scripts/component-binding-view-evidence.ts "$evidence" --correctness
