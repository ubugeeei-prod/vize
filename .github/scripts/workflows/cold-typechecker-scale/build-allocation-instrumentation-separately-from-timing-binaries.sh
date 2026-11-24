#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/cold-typechecker-scale/build-allocation-instrumentation-separately-from-timing-binaries.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
for source in base head; do
  source_root="$GITHUB_WORKSPACE"
  if [[ "$source" == base ]]; then source_root="$RUNNER_TEMP/base"; fi
  node tools/benchmarks/scripts/cold-typechecker-scale-build.ts --source-root "$source_root" --target-dir "$RUNNER_TEMP/cold-builds/profile-$source" --binary "$RUNNER_TEMP/vize-profile-$source" --receipt "$RUNNER_TEMP/cold-scale/builds/profile-$source.json" --role "profile-$source" --allocation
done
node tools/benchmarks/scripts/cold-typechecker-scale-build.ts --base-receipt "$RUNNER_TEMP/cold-scale/builds/profile-base.json" --head-receipt "$RUNNER_TEMP/cold-scale/builds/profile-head.json" --receipt "$RUNNER_TEMP/cold-scale/builds/profile-pair.json"
