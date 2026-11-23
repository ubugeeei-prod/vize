#!/usr/bin/env bash
set -euo pipefail
mkdir -p target/differential
node --test tests/tooling/formatter-expression-width-semantics.test.mjs tests/tooling/formatter-continuation-prefix-width.test.mjs 2>&1 | tee target/differential/formatter-source.log
