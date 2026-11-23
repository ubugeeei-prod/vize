#!/usr/bin/env bash
set -euo pipefail
npm install --prefix "$RUNNER_TEMP/typed-editor" --ignore-scripts --no-audit --no-fund --save-exact @typescript/typescript-linux-x64@7.0.2
export CORSA_PATH="$RUNNER_TEMP/typed-editor/node_modules/@typescript/typescript-linux-x64/lib/tsc"
"$CORSA_PATH" --version
cargo build --locked --profile ci-opt -p vize
node tools/support/compat/davinci/literal-hover-oracle.ts target/ci-opt/vize "$RUNNER_TEMP/original-hovers.json"
