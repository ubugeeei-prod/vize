#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/nuxt3-module-build/install-the-frozen-nuxt-5-development-reproduction.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
mkdir -p "$RUNNER_TEMP/nuxt5-dev-runtime-proof"
npm ci --legacy-peer-deps --no-audit --no-fund 2>&1 | tee "$RUNNER_TEMP/nuxt5-dev-runtime-proof/install.log"
