#!/usr/bin/env bash
mkdir -p "$RUNNER_TEMP/defaulted-inject-proof"
npm view --registry=https://registry.npmjs.org vize@0.602.0 --json > "$RUNNER_TEMP/defaulted-inject-proof/before-registry.json"
npm install --registry=https://registry.npmjs.org --prefix "$RUNNER_TEMP/defaulted-inject-before" --ignore-scripts --no-audit --no-fund vize@0.602.0 > "$RUNNER_TEMP/defaulted-inject-proof/install.stdout" 2> "$RUNNER_TEMP/defaulted-inject-proof/install.stderr"
cp "$RUNNER_TEMP/defaulted-inject-before/package-lock.json" "$RUNNER_TEMP/defaulted-inject-proof/before-lock.json"
cp target/ci/vize.differential-build.json "$RUNNER_TEMP/defaulted-inject-proof/candidate-build.json"
cp target/ci/vize.source-use.json "$RUNNER_TEMP/defaulted-inject-proof/candidate-use.json"
