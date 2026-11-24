#!/usr/bin/env bash
set -euo pipefail
git fetch --no-tags --depth=1 origin "$BASE_SHA" "$HEAD_SHA"
VIZE_CODEGEN_PROBE=1 node --test tests/tooling/native-addon-identity.test.ts
node --test tests/tooling/stock-addon-qualification.test.ts
receipt="$RUNNER_TEMP/stock-addon-qualification.json"
mode=$(node tools/support/compat/npm/stock-addon-qualification.ts "$BASE_SHA" "$HEAD_SHA" "$receipt")
control=()
if [[ "$mode" = byte-identity ]]; then control=(--control-plugin-identity); fi
status=0
node tools/commands/levels/codegen-equivalence.ts "$BASE_SHA" "$HEAD_SHA" vize_vitrine node-addon default "$RUNNER_TEMP/stock-addon.json" "${control[@]}" || status=$?
# The real builds stamp their respective Git SHA and source closure into the plugin cache key.
test "$status" = 1
if [[ "$mode" = byte-identity ]]; then
  node tools/commands/levels/native-addon-identity.ts "$RUNNER_TEMP/stock-addon.json" | tee "$RUNNER_TEMP/stock-addon-identity.json"
  node tools/support/compat/npm/bundler-block-products.ts --stock "$RUNNER_TEMP/stock-addon.json"
else
  test "$mode" = stock-runtime
  node tools/support/compat/npm/bundler-block-products.ts --stock-runtime "$RUNNER_TEMP/stock-addon.json" "$receipt"
fi
