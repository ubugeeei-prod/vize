#!/usr/bin/env bash
provider_prefix="$RUNNER_TEMP/vue-benchmarks-current-provider"
node tools/benchmarks/scripts/vue-benchmarks-current-typecheck.mjs \
  --source-sha "$EXPECTED_SOURCE_SHA" \
  --vize-bin "$GITHUB_WORKSPACE/target/ci/vize" \
  --corsa-bin "$provider_prefix/node_modules/@typescript/typescript-linux-x64/lib/tsc" \
  --vue-dir "$provider_prefix/node_modules/vue" \
  --vue-tsc-bin "$provider_prefix/node_modules/.bin/vue-tsc" \
  --output "$RUNNER_TEMP/vue-benchmarks-current-typecheck"
