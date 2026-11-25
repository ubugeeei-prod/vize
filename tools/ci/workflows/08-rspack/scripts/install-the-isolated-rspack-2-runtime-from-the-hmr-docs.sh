#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/08-rspack/scripts/install-the-isolated-rspack-2-runtime-from-the-hmr-docs.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
runtime="${RUNNER_TEMP}/vize-rspack-hmr-runtime"
npm install --prefix "$runtime" --legacy-peer-deps \
  @rspack/core@2.2.2 @rspack/dev-server@2.2.1 \
  vue@3.5.42 playwright@1.62.1 css-loader@7.1.2
vize-ci-apt-retry node "$runtime/node_modules/playwright/cli.js" install --with-deps chromium
echo "VIZE_HMR_RUNTIME=$runtime" >> "$GITHUB_ENV"
