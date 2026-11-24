#!/usr/bin/env bash
"$VIZE_NUXT_SOURCE_NODE" tools/support/compat/nuxt/nuxt5-runtime-build.ts "$RUNNER_TEMP/nuxt5-runtime-proof/acceptance" "$(command -v node)"
