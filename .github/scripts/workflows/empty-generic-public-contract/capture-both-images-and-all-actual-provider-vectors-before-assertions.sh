#!/usr/bin/env bash
node tools/support/compat/content-mapper/empty-generic-contract.ts \
  --consumer "$RUNNER_TEMP/empty-generic-consumer" \
  --baseline "$RUNNER_TEMP/empty-generic-target-base/ci/vize" \
  --candidate "$RUNNER_TEMP/empty-generic-target-head/ci/vize" \
  --baseline-source "$BASE_SHA" --source "$SOURCE_SHA" \
  --output "$RUNNER_TEMP/empty-generic-evidence"
