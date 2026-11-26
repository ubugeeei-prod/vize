#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/08-nuxt3-module/scripts/verify-source-bound-nuxt-5-development-stylesheets-and-hydration.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
"$VIZE_NUXT_SOURCE_NODE" tools/support/compat/nuxt/nuxt5-dev-runtime-build.ts "$RUNNER_TEMP/nuxt5-dev-runtime-proof/acceptance" "$(command -v node)"
