#!/usr/bin/env bash
vp node tests/tooling/support/n8n-compiler-custody.mjs \
  "$VIZE_N8N_CAPTURE_BASELINE" \
  "$VIZE_N8N_REPAIR_ANCHOR" \
  tests/_fixtures/_git/n8n \
  "target/n8n-compiler-custody/${GITHUB_RUN_ID}-${GITHUB_RUN_ATTEMPT}" \
  "$RUNNER_TEMP/n8n-official-compiler.mjs"
vp node "$VIZE_CI_FILES/build-and-capture-both-authenticated-source-states-with-the-same-recipe.ts" "target/n8n-compiler-custody/${GITHUB_RUN_ID}-${GITHUB_RUN_ATTEMPT}/capture.json"
