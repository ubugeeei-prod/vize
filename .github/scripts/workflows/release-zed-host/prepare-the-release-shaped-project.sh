#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/release-zed-host/prepare-the-release-shaped-project.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
project="${RUNNER_TEMP}/release-zed-project"
evidence="${RUNNER_TEMP}/release-zed-evidence"
mkdir -p "$evidence"
cp -R editors/vscode/test-fixtures/extension-host/real-vue "$project"
node "$VIZE_CI_FILES/prepare-the-release-shaped-project.ts" "$project"
npm install --prefix "$project" --ignore-scripts --no-audit --no-fund \
  --package-lock=false --save=false vue@3.5.34 typescript@6.0.3
target/ci/vize lint "$project/src/Lint.vue" --format json --no-config \
  > "$evidence/source-diagnostic.json" || test "$?" -eq 1
