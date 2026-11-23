#!/usr/bin/env bash
set -euo pipefail
node tools/commands/levels/codegen-equivalence.ts \
  "$BASE_SHA" "$HEAD_SHA" "$PACKAGE" "$PRODUCT" "$MODE" "$RUNNER_TEMP/codegen-$MODE.json" | tee -a "$GITHUB_STEP_SUMMARY"
