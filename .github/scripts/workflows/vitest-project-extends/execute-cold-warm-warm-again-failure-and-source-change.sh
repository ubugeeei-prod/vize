#!/usr/bin/env bash
node npm/native/scripts/test-preparation.mjs node tools/support/compat/vitest-module-cache.ts \
  --candidate npm/builder/vite/dist/index.mjs \
  --output target/vitest-module-cache
