#!/usr/bin/env bash
mkdir -p "$RUNNER_TEMP/router-navigation-checker"
cp tests/fixtures/typechecker/experimental-router-navigation-5.4/checker-host-package.json.txt "$RUNNER_TEMP/router-navigation-checker/package.json"
cp tests/fixtures/typechecker/experimental-router-navigation-5.4/checker-host-package-lock.json.txt "$RUNNER_TEMP/router-navigation-checker/package-lock.json"
npm ci --prefix "$RUNNER_TEMP/router-navigation-checker" --ignore-scripts --no-audit --no-fund --prefer-offline --registry=https://registry.npmjs.org
sha256sum "$RUNNER_TEMP/router-navigation-checker/package-lock.json" "$RUNNER_TEMP/router-navigation-checker/node_modules/@typescript/typescript-linux-x64/lib/tsc" > "$RUNNER_TEMP/router-navigation/checker-identity.sha256"
