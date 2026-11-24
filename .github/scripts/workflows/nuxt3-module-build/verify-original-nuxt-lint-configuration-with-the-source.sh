#!/usr/bin/env bash
vp env exec node tools/support/compat/nuxt/lint-config-build.ts "$RUNNER_TEMP/nuxt-lint-config-proof" "$RUNNER_TEMP/nuxt3-module-proof/source-native/custody.json"
