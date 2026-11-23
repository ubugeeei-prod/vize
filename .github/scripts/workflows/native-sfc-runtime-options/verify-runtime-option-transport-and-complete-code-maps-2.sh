#!/usr/bin/env bash
set -euo pipefail
mkdir -p "$VIZE_NATIVE_SFC_RUNTIME_OPTIONS_PROOF"
node --experimental-vm-modules --test tests/tooling/native-sfc-runtime-options.test.mjs 2>&1 | tee "$VIZE_NATIVE_SFC_RUNTIME_OPTIONS_PROOF/tap.log"
