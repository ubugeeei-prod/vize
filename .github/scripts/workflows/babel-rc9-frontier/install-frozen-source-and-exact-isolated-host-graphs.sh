#!/usr/bin/env bash
vp install --frozen-lockfile --prefer-offline --filter './npm/builder/babel...' --filter './npm/native...'
npm ci --prefix npm/builder/babel/test/hosts/fixtures/babel7 --ignore-scripts --no-audit --no-fund
npm ci --prefix npm/builder/babel/test/hosts/fixtures/babel8 --ignore-scripts --no-audit --no-fund
for runtime in vdom vapor-rc9 vapor-rc11; do
  npm ci --prefix "npm/builder/babel/test/runtime/fixtures/$runtime" --ignore-scripts --no-audit --no-fund
done
node npm/builder/babel/test/runtime/fixtures/vdom/node_modules/playwright/cli.js install --with-deps chromium
