#!/usr/bin/env bash
node "$RUNNER_TEMP/native-ops-harness/native-ops-driver.ts" publish \
  "$RUNNER_TEMP/native-ops-private" "$RUNNER_TEMP/native-ops-public" \
  "$RUNNER_TEMP/js-native-fault-evidence-*" \
  "npm/native/.artifacts/native/formatter-history" \
  "target/template-style-fixes-7905.json" \
  "target/oxlint-original-locations-7904.json" \
  "target/oxlint-original-project-checks-7903.json" \
  "target/oxlint-script-safe-carrier-7903.json" \
  "target/oxlint-original-html-binding-7903" \
  "target/vize-tests/original-sfc-host-7004-7005-*"
