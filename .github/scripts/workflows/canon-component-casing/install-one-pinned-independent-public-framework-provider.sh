#!/usr/bin/env bash
set -euo pipefail
mkdir "$RUNNER_TEMP/component-casing-provider"
cd "$RUNNER_TEMP/component-casing-provider"
node "$VIZE_CI_FILES/install-one-pinned-independent-public-framework-provider.ts"
npm install --ignore-scripts --no-audit --no-fund
