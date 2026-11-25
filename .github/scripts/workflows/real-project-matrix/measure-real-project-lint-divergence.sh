#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/real-project-matrix/measure-real-project-lint-divergence.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
/usr/bin/time --verbose --output "$FIXTURE_REPORT_DIR/lint-divergence-resource.txt" \
  node tools/commands/fixtures/lint-divergence-report.ts \
  --shard-index "$FIXTURE_SHARD_INDEX" --shard-count "$FIXTURE_SHARD_COUNT" \
  --all-mapped-rules --measure-coverage-gap --budget-mode "$LINT_DIVERGENCE_MODE" \
  --vize-bin target/ci/vize --timeout-ms 600000 \
  --output-dir "$FIXTURE_REPORT_DIR"
test -s "$FIXTURE_REPORT_DIR/lint-divergence-summary.json"
LC_ALL=C sha256sum "$FIXTURE_REPORT_DIR"/*-lint-divergence.json \
  > "$FIXTURE_REPORT_DIR/lint-divergence-hashes.txt"
