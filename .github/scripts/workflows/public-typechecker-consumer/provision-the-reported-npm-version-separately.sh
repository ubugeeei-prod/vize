#!/usr/bin/env bash
set -euo pipefail
npm install --prefix "$RUNNER_TEMP/11317-manager" npm@11.19.1 --ignore-scripts --no-audit --no-fund
test "$(node "$RUNNER_TEMP/11317-manager/node_modules/npm/bin/npm-cli.js" --version)" = 11.19.1
