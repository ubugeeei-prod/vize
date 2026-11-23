#!/usr/bin/env bash
mkdir -p "$RUNNER_TEMP/nuxt5-dev-runtime-proof"
npm ci --legacy-peer-deps --no-audit --no-fund 2>&1 | tee "$RUNNER_TEMP/nuxt5-dev-runtime-proof/install.log"
