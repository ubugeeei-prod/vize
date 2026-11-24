#!/usr/bin/env bash
vp env exec node tools/support/compat/nuxt/compiler-defaults-build.ts "$RUNNER_TEMP/nuxt-compiler-defaults-proof" "$RUNNER_TEMP/nuxt3-module-proof/source-native/custody.json"
