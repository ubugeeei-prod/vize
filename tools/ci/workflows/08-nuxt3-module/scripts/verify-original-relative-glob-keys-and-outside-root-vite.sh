#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/08-nuxt3-module/scripts/verify-original-relative-glob-keys-and-outside-root-vite.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
vp env exec node tools/support/compat/nuxt/relative-glob-build.ts "$RUNNER_TEMP/vite-relative-glob-proof" "$RUNNER_TEMP/nuxt3-module-proof/source-native/custody.json"
