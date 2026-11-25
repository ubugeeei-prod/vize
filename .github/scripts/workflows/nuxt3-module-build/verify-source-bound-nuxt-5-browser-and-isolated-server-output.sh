#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/nuxt3-module-build/verify-source-bound-nuxt-5-browser-and-isolated-server-output.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
"$VIZE_NUXT_SOURCE_NODE" tools/support/compat/nuxt/nuxt5-runtime-build.ts "$RUNNER_TEMP/nuxt5-runtime-proof/acceptance" "$(command -v node)"
