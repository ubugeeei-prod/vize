#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/cold-typechecker-scale/build-normal-exact-base-and-head-with-equal-compiler-settings.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
for role in base head; do
  node tools/benchmarks/scripts/cold-typechecker-scale-build.ts --source-root "$RUNNER_TEMP/cold-paired-$role" --target-dir "$RUNNER_TEMP/cold-paired-builds/$role" --binary "$RUNNER_TEMP/cold-paired/binaries/$role/vize" --receipt "$RUNNER_TEMP/cold-paired/builds/$role.json" --role "normal-$role"
done
node tools/benchmarks/scripts/cold-typechecker-scale-build.ts --base-receipt "$RUNNER_TEMP/cold-paired/builds/base.json" --head-receipt "$RUNNER_TEMP/cold-paired/builds/head.json" --receipt "$RUNNER_TEMP/cold-paired/builds/pair.json"
