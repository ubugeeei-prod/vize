set -uo pipefail

failed=0
for sdk in actual-20261024 beta-3e923 d6c4; do
  extra=()
  if [[ "$sdk" == beta-3e923 ]]; then
    native="$RUNNER_TEMP/component-input-beta/node_modules/@typescript/typescript-linux-x64/lib/tsc"
    extra=(--mapper-host "$native" --mapper-version 'Version 7.1.0-beta')
  elif [[ "$sdk" == d6c4 ]]; then
    native="$RUNNER_TEMP/component-input-d6c4/tsgo"
    extra=(--mapper-host "$native" --mapper-version "$("$native" --version)")
  fi
  node tools/support/compat/content-mapper/diagnostic-ownership.ts \
    --callback-cfa-controls \
    --consumer "$RUNNER_TEMP/component-input-consumer" \
    --candidate-cli "$RUNNER_TEMP/component-input-ownership/source-images/candidate-vize" \
    --source-sha "$SOURCE_SHA" \
    --comparison-cli "$RUNNER_TEMP/component-input-ownership/source-images/base-vize" \
    --comparison-source-sha "$BASE_SHA" \
    --historical-comparison-source-sha "$HISTORICAL_CALLBACK_BEFORE" \
    --comparison-role pull-request-base \
    --output "$RUNNER_TEMP/component-input-ownership/callback-cfa/$sdk" "${extra[@]}" || failed=1
done
exit "$failed"
