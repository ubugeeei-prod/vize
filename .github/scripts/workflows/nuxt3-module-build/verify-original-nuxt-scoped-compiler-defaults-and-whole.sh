#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/nuxt3-module-build/verify-original-nuxt-scoped-compiler-defaults-and-whole.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
vp env exec node tools/support/compat/nuxt/compiler-defaults-build.ts "$RUNNER_TEMP/nuxt-compiler-defaults-proof" "$RUNNER_TEMP/nuxt3-module-proof/source-native/custody.json"
