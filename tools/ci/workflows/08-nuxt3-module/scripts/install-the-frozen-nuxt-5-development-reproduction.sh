#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/08-nuxt3-module/scripts/install-the-frozen-nuxt-5-development-reproduction.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
mkdir -p "$RUNNER_TEMP/nuxt5-dev-runtime-proof"
npm ci --legacy-peer-deps --no-audit --no-fund 2>&1 | tee "$RUNNER_TEMP/nuxt5-dev-runtime-proof/install.log"
