#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/01-real-project-matrix/scripts/enforce-typechecker-baseline-divergence.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
rust-script tools/commands/fixtures/typecheck-divergence-report.rs \
  --report-dir "$FIXTURE_REPORT_DIR" \
  --shard-index "$FIXTURE_SHARD_INDEX" \
  --shard-count "$FIXTURE_SHARD_COUNT" \
  --budget-mode "$BUDGET_MODE" \
  --vize-bin target/ci/vize \
  --vue-tsc-bin tests/node_modules/.bin/vue-tsc
