#!/usr/bin/env bash
set -euo pipefail
test "$(git rev-parse HEAD)" = "$SOURCE_SHA"
mkdir -p "$RUNNER_TEMP/babel-plugin/packed"
git rev-parse HEAD HEAD^{tree} > "$RUNNER_TEMP/babel-plugin/source.txt"
git log -50 --format='%H %T %P %s' > "$RUNNER_TEMP/babel-plugin/ordered-history.txt"
node --version > "$RUNNER_TEMP/babel-plugin/node-version.txt"
vp run --workspace-root build:native:test
GITHUB_SHA="$SOURCE_SHA" node npm/native/scripts/native-build-receipt.mjs
vp run --no-cache --filter './npm/builder/babel' build
(cd npm/builder/babel && npm pack --ignore-scripts --pack-destination "$RUNNER_TEMP/babel-plugin/packed" --json) > "$RUNNER_TEMP/babel-plugin/pack.json"
sha256sum "$RUNNER_TEMP/babel-plugin/packed/"*.tgz > "$RUNNER_TEMP/babel-plugin/packed.sha256"
cp npm/native/.artifacts/native/build-token-receipt.json "$RUNNER_TEMP/babel-plugin/"
