#!/usr/bin/env bash
set -euo pipefail
mkdir -p "$VIZE_NATIVE_SFC_VERSION_PROOF"
node --test tests/tooling/native-sfc-runtime-version.test.mjs 2>&1 | tee "$VIZE_NATIVE_SFC_VERSION_PROOF/tap.log"
