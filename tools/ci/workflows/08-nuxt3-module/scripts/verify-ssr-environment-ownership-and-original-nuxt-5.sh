#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/08-nuxt3-module/scripts/verify-ssr-environment-ownership-and-original-nuxt-5.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
node --test \
  npm/framework/nuxt/src/ssr-runtime.test.ts \
  tests/tooling/nuxt5-runtime.test.ts \
  tests/tooling/nuxt-source-binding.test.mjs
