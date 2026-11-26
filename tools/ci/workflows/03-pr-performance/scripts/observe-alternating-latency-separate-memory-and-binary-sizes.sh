#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/03-pr-performance/scripts/observe-alternating-latency-separate-memory-and-binary-sizes.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
node driver/tools/benchmarks/scripts/pr-performance-observation.ts \
  --base-bin "$RUNNER_TEMP/pr-performance-base-$GITHUB_RUN_ID-$GITHUB_RUN_ATTEMPT/ci-opt/vize" \
  --head-bin "$RUNNER_TEMP/pr-performance-head-$GITHUB_RUN_ID-$GITHUB_RUN_ATTEMPT/ci-opt/vize" \
  --base-source "$BASE_SHA" --head-source "$HEAD_SHA" \
  --input-dir "$OBSERVATION_INPUT" --vue-dir "$OBSERVATION_VUE" \
  --corsa-path "$OBSERVATION_CORSA" --runs 5 --warmups 1 \
  --out "$GITHUB_WORKSPACE/performance-observation/measurements" \
  --build-receipts "$GITHUB_WORKSPACE/performance-observation/build"
