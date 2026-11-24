#!/usr/bin/env bash
node "$RUNNER_TEMP/native-ops-harness/native-ops-driver.ts" original \
  "$RUNNER_TEMP/native-ops-private/original" \
  "$RUNNER_TEMP/native-ops-private/control/control-complete.json"
