#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/nuxt3-module-build/verify-real-javascript-nuxt-and-vite-workspace-products.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
vp env exec node tools/support/compat/nuxt/javascript-workspace-build.ts "$RUNNER_TEMP/javascript-workspace-proof" "$RUNNER_TEMP/nuxt3-module-proof/source-native/custody.json"
