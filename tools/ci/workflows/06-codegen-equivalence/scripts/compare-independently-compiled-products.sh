#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/06-codegen-equivalence/scripts/compare-independently-compiled-products.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
node tools/commands/levels/codegen-equivalence.ts \
  "$BASE_SHA" "$HEAD_SHA" "$PACKAGE" "$PRODUCT" "$MODE" "$RUNNER_TEMP/codegen-$MODE.json" | tee -a "$GITHUB_STEP_SUMMARY"
