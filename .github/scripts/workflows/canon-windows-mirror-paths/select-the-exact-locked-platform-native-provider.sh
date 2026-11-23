#!/usr/bin/env bash
set -euo pipefail
test "$(git rev-parse HEAD)" = "$EXPECTED_SOURCE"
node "$VIZE_CI_FILES/select-the-exact-locked-platform-native-provider.ts"
# The receipt describes this run only. Run-output pruning of a reused
# target is Linux-only, and a target restored from the Actions cache
# on Windows can still hold an earlier run's receipt, which the
# exclusive (`wx`) write would refuse.
rm -f target/nextest/full/native-runtime-provider.json
GITHUB_SHA="$EXPECTED_SOURCE" vp exec node tools/support/compat/github/full-rust-runtime.ts
