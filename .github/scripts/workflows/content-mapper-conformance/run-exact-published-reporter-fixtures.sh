#!/usr/bin/env bash
case "$PUBLIC_REPORT" in
  9499) public_fixture=tests/_fixtures/content-mapper/with-defaults-surplus.json ;;
  9498) public_fixture=tests/_fixtures/content-mapper/escaton-public-props-9498.json ;;
  *) echo "Unsupported published report" >&2; exit 2 ;;
esac
sha256sum "$public_fixture" >> "$RUNNER_TEMP/official-css-source/inputs.sha256"
node "$VIZE_CI_FILES/run-exact-published-reporter-fixtures.ts"
node tools/support/compat/content-mapper/official-css-options.ts --candidate-only --explicit-fixtures-only --public-version "${PUBLIC_TAG#v}" --public-inputs "$PUBLIC_INPUTS" --fixture "$public_fixture" --output "$RUNNER_TEMP/official-css-options"
