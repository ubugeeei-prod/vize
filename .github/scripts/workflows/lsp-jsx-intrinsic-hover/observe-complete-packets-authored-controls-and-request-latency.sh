#!/usr/bin/env bash
set -euo pipefail
node --test --test-concurrency=1 tests/tooling/lsp-jsx-intrinsic-hover.test.ts tests/tooling/differential-lsp-profiles.test.ts 2>&1 | tee target/vize-tests/jsx-intrinsic-hover/protocol.log
