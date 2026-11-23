#!/usr/bin/env bash
set -euo pipefail
mkdir "$RUNNER_TEMP/partition-provider"
node "$VIZE_CI_FILES/install-the-unchanged-upstream-provider-pins.cjs"
npm install --prefix "$RUNNER_TEMP/partition-provider" --ignore-scripts --no-audit --no-fund
