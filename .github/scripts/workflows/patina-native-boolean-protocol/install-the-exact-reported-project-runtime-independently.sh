#!/usr/bin/env bash
npm install --prefix "$RUNNER_TEMP/patina-tsrs" --ignore-scripts --no-audit --no-fund tsrs@0.20.1-ts7.1.0-beta
echo "VIZE_TEST_TSRS_PACKAGE=$RUNNER_TEMP/patina-tsrs/node_modules/tsrs" >> "$GITHUB_ENV"
mkdir -p "$GITHUB_WORKSPACE/target/patina-native-boolean"
cp "$RUNNER_TEMP/patina-tsrs/package-lock.json" "$GITHUB_WORKSPACE/target/patina-native-boolean/provider-package-lock.json"
sha256sum "$RUNNER_TEMP/patina-tsrs/node_modules/@tsrs/linux-x64-gnu/tsrs" "$GITHUB_WORKSPACE/node_modules/@typescript/typescript-linux-x64/lib/tsc" > "$GITHUB_WORKSPACE/target/patina-native-boolean/provider-sha256.txt"
