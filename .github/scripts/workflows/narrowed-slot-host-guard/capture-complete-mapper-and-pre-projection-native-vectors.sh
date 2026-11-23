#!/usr/bin/env bash
node tools/support/compat/content-mapper/narrowed-slot-guard.ts \
  --consumer "$RUNNER_TEMP/narrowed-slot-consumer" \
  --candidate "$GITHUB_WORKSPACE/target/ci/vize" \
  --source "$SOURCE_SHA" \
  --output "$RUNNER_TEMP/narrowed-slot-evidence"
