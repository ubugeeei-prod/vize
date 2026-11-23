#!/usr/bin/env bash
mkdir -p "$RUNNER_TEMP/cross-file-offset-proof"
npm view --registry=https://registry.npmjs.org vize@0.601.0 --json > "$RUNNER_TEMP/cross-file-offset-proof/before-registry.json"
npm install --registry=https://registry.npmjs.org --prefix "$RUNNER_TEMP/cross-file-before" --ignore-scripts --no-audit --no-fund vize@0.601.0 > "$RUNNER_TEMP/cross-file-offset-proof/before-install.stdout" 2> "$RUNNER_TEMP/cross-file-offset-proof/before-install.stderr"
cp "$RUNNER_TEMP/cross-file-before/package-lock.json" "$RUNNER_TEMP/cross-file-offset-proof/before-lock.json"
cp "$RUNNER_TEMP/cross-file-before/node_modules/vize/package.json" "$RUNNER_TEMP/cross-file-offset-proof/before-manifest.json"
cp target/ci/vize.differential-build.json "$RUNNER_TEMP/cross-file-offset-proof/candidate-build.json"
cp target/ci/vize.source-use.json "$RUNNER_TEMP/cross-file-offset-proof/candidate-use.json"
