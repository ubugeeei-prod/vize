#!/usr/bin/env bash
set -euo pipefail
test ! -e npm/cli/node_modules
ln -s "$RUNNER_TEMP/slot-provider-reference/node_modules" npm/cli/node_modules
native=$(node "$VIZE_CI_FILES/compare-actual-source-declaration-consumers-against-the-frozen-c3-preima.ts" "$RUNNER_TEMP/slot-provider-reference/node_modules/typescript-native/lib/getExePath.js")
VIZE_PUBLIC_SLOT_NATIVE="$native" VIZE_PUBLIC_SLOT_RECEIPT="$RUNNER_TEMP/slot-provider-any/public-instance.json" \
  node --test --test-name-pattern='public Instance retains' tests/tooling/content-mapper-shared-types.test.ts \
  > "$RUNNER_TEMP/slot-provider-any/public-instance.stdout" 2> "$RUNNER_TEMP/slot-provider-any/public-instance.stderr"
