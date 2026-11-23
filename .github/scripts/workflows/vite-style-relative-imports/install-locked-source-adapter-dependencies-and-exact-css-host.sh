#!/usr/bin/env bash
vp install --frozen-lockfile --prefer-offline --filter './npm/builder/vite...' --filter './npm/cli...' --filter './npm/native...'
npm ci --prefix tests/_fixtures/_projects/vize-style-relative-imports-host --ignore-scripts --no-audit --no-fund
if [ -e node_modules/lightningcss ] || [ -L node_modules/lightningcss ]; then
  mv node_modules/lightningcss "$RUNNER_TEMP/vize-original-lightningcss-link"
fi
ln -s "$GITHUB_WORKSPACE/tests/_fixtures/_projects/vize-style-relative-imports-host/node_modules/lightningcss" node_modules/lightningcss
