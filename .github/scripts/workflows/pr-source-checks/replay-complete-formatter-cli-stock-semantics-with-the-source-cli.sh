#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/pr-source-checks/replay-complete-formatter-cli-stock-semantics-with-the-source-cli.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
mkdir -p target/differential
node --test tests/tooling/formatter-expression-width-semantics.test.mjs tests/tooling/formatter-continuation-prefix-width.test.mjs 2>&1 | tee target/differential/formatter-source.log
