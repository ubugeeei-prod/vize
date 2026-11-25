#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/vite-precompile-cache/verify-in-memory-persistent-and-ssr-precompile.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
node --test --test-concurrency=1 \
  npm/builder/vite/src/plugin/compiled-module-cache.test.ts \
  npm/builder/vite/src/plugin/precompile-cache-path.test.ts \
  npm/builder/vite/src/plugin/precompile.test.ts
