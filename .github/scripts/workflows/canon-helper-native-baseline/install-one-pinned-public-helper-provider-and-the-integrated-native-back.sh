#!/usr/bin/env bash
set -euo pipefail
mkdir "$RUNNER_TEMP/helper-provider"
cd "$RUNNER_TEMP/helper-provider"
node "$VIZE_CI_FILES/install-one-pinned-public-helper-provider-and-the-integrated-native-back.ts"
npm install --ignore-scripts --no-audit --no-fund
cp package.json package-lock.json "$RUNNER_TEMP/helper-baseline-custody/"
