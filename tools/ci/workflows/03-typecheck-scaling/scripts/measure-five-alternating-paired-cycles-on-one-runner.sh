#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/03-typecheck-scaling/scripts/measure-five-alternating-paired-cycles-on-one-runner.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
node driver/tools/benchmarks/scripts/typecheck-scaling-paired.ts \
  --base-bin "$RUNNER_TEMP/paired-scaling-base-$GITHUB_RUN_ID-$GITHUB_RUN_ATTEMPT/ci-opt/vize" \
  --head-bin "$RUNNER_TEMP/paired-scaling-head-$GITHUB_RUN_ID-$GITHUB_RUN_ATTEMPT/ci-opt/vize" \
  --base-source "$BASE_SHA" --head-source "$HEAD_SHA" \
  --provider-root "$GITHUB_WORKSPACE/driver" \
  --build-receipts "$GITHUB_WORKSPACE/paired-scaling/build" \
  --sizes "$SIZES" --runs 5 \
  --runner blacksmith-32vcpu-ubuntu-2404 \
  --run-url "$GITHUB_SERVER_URL/$GITHUB_REPOSITORY/actions/runs/$GITHUB_RUN_ID" \
  --work "$RUNNER_TEMP/paired-scaling-work" \
  --out "$GITHUB_WORKSPACE/paired-scaling/measurements"
