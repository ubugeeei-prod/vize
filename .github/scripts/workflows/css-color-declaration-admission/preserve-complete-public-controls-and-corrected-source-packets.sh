#!/usr/bin/env bash
set -euo pipefail
mkdir -p target/vize-tests/css-color-declaration-admission
node --test tests/tooling/lsp-css-color-declaration-admission.test.ts 2>&1 | tee target/vize-tests/css-color-declaration-admission/protocol.log
