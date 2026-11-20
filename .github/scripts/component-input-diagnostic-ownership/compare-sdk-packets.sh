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
    --consumer "$RUNNER_TEMP/component-input-consumer" \
    --candidate-cli "$GITHUB_WORKSPACE/target/ci/vize" \
    --source-sha "$SOURCE_SHA" \
    --output "$RUNNER_TEMP/component-input-ownership/$sdk" "${extra[@]}" || failed=1

  if [[ "$sdk" == actual-20261024 ]]; then
    aria_host="$(node --input-type=module -e 'const {default: resolve} = await import(`${process.env.RUNNER_TEMP}/component-input-consumer/node_modules/typescript/lib/getExePath.js`); console.log(resolve());')"
  else
    aria_host="$native"
  fi
  node tools/support/compat/content-mapper/aria-component-attrs.ts \
    --consumer "$RUNNER_TEMP/component-input-consumer" \
    --candidate-cli "$GITHUB_WORKSPACE/target/ci/vize" \
    --source-sha "$SOURCE_SHA" \
    --mapper-host "$aria_host" \
    --mapper-version "$("$aria_host" --version)" \
    --output "$RUNNER_TEMP/component-input-ownership/aria-package/$sdk" || failed=1
done
exit "$failed"
