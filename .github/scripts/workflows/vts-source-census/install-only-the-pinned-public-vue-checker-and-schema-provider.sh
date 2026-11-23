#!/usr/bin/env bash
set -euo pipefail
mkdir "$RUNNER_TEMP/census-provider"
cd "$RUNNER_TEMP/census-provider"
node "$VIZE_CI_FILES/install-only-the-pinned-public-vue-checker-and-schema-provider.cjs"
npm install --ignore-scripts --no-audit --no-fund
