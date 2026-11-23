#!/usr/bin/env bash
node tools/support/compat/fixtures/tool-matrix-report.ts \
  --shard-index "$FIXTURE_SHARD_INDEX" \
  --shard-count "$FIXTURE_SHARD_COUNT" \
  --vize-bin target/ci/vize \
  --timeout-ms "$CORE_TOOLS_TIMEOUT_MS" \
  --output-dir "$FIXTURE_REPORT_DIR"
