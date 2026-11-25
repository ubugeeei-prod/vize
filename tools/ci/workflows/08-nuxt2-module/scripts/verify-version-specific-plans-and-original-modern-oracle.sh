#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/08-nuxt2-module/scripts/verify-version-specific-plans-and-original-modern-oracle.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
vp node \
  --test npm/framework/nuxt-lint-config/src/*.test.ts npm/framework/nuxt/src/lint/generation*.test.ts npm/framework/nuxt/src/builder.test.ts npm/framework/nuxt/src/lint/checker/setup.test.ts
