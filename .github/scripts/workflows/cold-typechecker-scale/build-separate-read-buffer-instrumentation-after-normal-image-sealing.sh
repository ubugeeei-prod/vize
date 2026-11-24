#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/cold-typechecker-scale/build-separate-read-buffer-instrumentation-after-normal-image-sealing.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
for role in base head; do
  source_sha="$BASE_SHA"
  if [[ "$role" == head ]]; then source_sha="$HEAD_SHA"; fi
  node tools/benchmarks/scripts/cold-typechecker-scale-read-buffer.ts "$RUNNER_TEMP/cold-paired-$role" "$source_sha" "$RUNNER_TEMP/cold-read-probe-$role" "$RUNNER_TEMP/cold-paired/instrumented/$role"
done
