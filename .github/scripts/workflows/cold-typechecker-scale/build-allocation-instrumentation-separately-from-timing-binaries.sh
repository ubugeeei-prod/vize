#!/usr/bin/env bash
for source in base head; do
  source_root="$GITHUB_WORKSPACE"
  if [[ "$source" == base ]]; then source_root="$RUNNER_TEMP/base"; fi
  node tools/benchmarks/scripts/cold-typechecker-scale-build.ts --source-root "$source_root" --target-dir "$RUNNER_TEMP/cold-builds/profile-$source" --binary "$RUNNER_TEMP/vize-profile-$source" --receipt "$RUNNER_TEMP/cold-scale/builds/profile-$source.json" --role "profile-$source" --allocation
done
node tools/benchmarks/scripts/cold-typechecker-scale-build.ts --base-receipt "$RUNNER_TEMP/cold-scale/builds/profile-base.json" --head-receipt "$RUNNER_TEMP/cold-scale/builds/profile-head.json" --receipt "$RUNNER_TEMP/cold-scale/builds/profile-pair.json"
