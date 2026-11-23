#!/usr/bin/env bash
set -euo pipefail
mkdir "$RUNNER_TEMP/slot-provider-reference"
cd "$RUNNER_TEMP/slot-provider-reference"
node "$VIZE_CI_FILES/install-pinned-independent-framework-and-native-compilers.ts"
npm install --ignore-scripts --no-audit --no-fund
cp package.json package-lock.json "$RUNNER_TEMP/slot-provider-any/"
