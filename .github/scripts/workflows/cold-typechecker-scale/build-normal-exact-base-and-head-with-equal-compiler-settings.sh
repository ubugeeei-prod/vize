#!/usr/bin/env bash
set -euo pipefail
for role in base head; do
  node tools/benchmarks/scripts/cold-typechecker-scale-build.ts --source-root "$RUNNER_TEMP/cold-paired-$role" --target-dir "$RUNNER_TEMP/cold-paired-builds/$role" --binary "$RUNNER_TEMP/cold-paired/binaries/$role/vize" --receipt "$RUNNER_TEMP/cold-paired/builds/$role.json" --role "normal-$role"
done
node tools/benchmarks/scripts/cold-typechecker-scale-build.ts --base-receipt "$RUNNER_TEMP/cold-paired/builds/base.json" --head-receipt "$RUNNER_TEMP/cold-paired/builds/head.json" --receipt "$RUNNER_TEMP/cold-paired/builds/pair.json"
