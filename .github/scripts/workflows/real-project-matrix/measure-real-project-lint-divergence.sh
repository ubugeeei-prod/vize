#!/usr/bin/env bash
/usr/bin/time --verbose --output "$FIXTURE_REPORT_DIR/lint-divergence-resource.txt" \
  node tools/commands/fixtures/lint-divergence-report.ts \
  --shard-index "$FIXTURE_SHARD_INDEX" --shard-count "$FIXTURE_SHARD_COUNT" \
  --all-mapped-rules --measure-coverage-gap --budget-mode "$LINT_DIVERGENCE_MODE" \
  --vize-bin target/ci/vize --timeout-ms 600000 \
  --output-dir "$FIXTURE_REPORT_DIR"
test -s "$FIXTURE_REPORT_DIR/lint-divergence-summary.json"
LC_ALL=C sha256sum "$FIXTURE_REPORT_DIR"/*-lint-divergence.json \
  > "$FIXTURE_REPORT_DIR/lint-divergence-hashes.txt"
