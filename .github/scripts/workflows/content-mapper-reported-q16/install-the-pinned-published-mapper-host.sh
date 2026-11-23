#!/usr/bin/env bash
mkdir -p "$RUNNER_TEMP/router-mapper-host"
cp tests/fixtures/typechecker/experimental-router-mapper-host/package.json.txt "$RUNNER_TEMP/router-mapper-host/package.json"
cp tests/fixtures/typechecker/experimental-router-mapper-host/package-lock.json.txt "$RUNNER_TEMP/router-mapper-host/package-lock.json"
npm ci --prefix "$RUNNER_TEMP/router-mapper-host" --ignore-scripts --no-audit --no-fund --prefer-offline --registry=https://registry.npmjs.org
sha256sum "$RUNNER_TEMP/router-mapper-host/package-lock.json" "$RUNNER_TEMP/router-mapper-host/node_modules/typescript/package.json" "$RUNNER_TEMP/router-mapper-host/node_modules/@typescript/typescript-linux-x64/lib/tsc" > "$RUNNER_TEMP/router-experimental/host-identity.sha256"
