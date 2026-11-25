#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/08-nuxt3-module/scripts/verify-nitro-plugin-ownership-and-frozen-development.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
node --test \
  npm/framework/nuxt/src/server-plugins.test.ts \
  npm/framework/nuxt/src/builder.test.ts \
  tests/tooling/nuxt5-dev-runtime.test.ts
