#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/babel-plugin-hosts/install-frozen-source-and-exact-isolated-host-graphs.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
node tools/support/github/vp-install.ts --filter './npm/builder/babel...' --filter './npm/native...'
npm ci --prefix npm/builder/babel/test/hosts/fixtures/babel7 --ignore-scripts --no-audit --no-fund
npm ci --prefix npm/builder/babel/test/hosts/fixtures/babel8 --ignore-scripts --no-audit --no-fund
for runtime in vdom vapor-rc9 vapor-rc11; do
  npm ci --prefix "npm/builder/babel/test/runtime/fixtures/$runtime" --ignore-scripts --no-audit --no-fund
done
node npm/builder/babel/test/runtime/fixtures/vdom/node_modules/playwright/cli.js install --with-deps chromium
