#!/usr/bin/env bash
node tools/support/compat/content-mapper/fetch-empty-generic-baseline.ts \
  > "$RUNNER_TEMP/empty-generic-baseline-fetch.json"
