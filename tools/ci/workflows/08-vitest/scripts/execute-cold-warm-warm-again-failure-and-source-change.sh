#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/08-vitest/scripts/execute-cold-warm-warm-again-failure-and-source-change.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
node npm/native/scripts/test-preparation.mjs node tools/support/compat/vitest-module-cache.ts \
  --candidate npm/builder/vite/dist/index.mjs \
  --output target/vitest-module-cache
