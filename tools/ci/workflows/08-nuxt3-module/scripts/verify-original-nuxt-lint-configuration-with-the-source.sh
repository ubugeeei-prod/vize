#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/08-nuxt3-module/scripts/verify-original-nuxt-lint-configuration-with-the-source.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
vp env exec node tools/support/compat/nuxt/lint-config-build.ts "$RUNNER_TEMP/nuxt-lint-config-proof" "$RUNNER_TEMP/nuxt3-module-proof/source-native/custody.json"
