#!/usr/bin/env bash
public_fixture=$(node "$VIZE_CI_FILES/select-published-fixture.ts" "$PUBLIC_REPORT")
sha256sum "$public_fixture" >> "$RUNNER_TEMP/official-css-source/inputs.sha256"
node "$VIZE_CI_FILES/run-exact-published-reporter-fixtures.ts"
node tools/support/compat/content-mapper/official-css-options.ts --candidate-only --explicit-fixtures-only --public-version "${PUBLIC_TAG#v}" --public-inputs "$PUBLIC_INPUTS" --fixture "$public_fixture" --output "$RUNNER_TEMP/official-css-options"
