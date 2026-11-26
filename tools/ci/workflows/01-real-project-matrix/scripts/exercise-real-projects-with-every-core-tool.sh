#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/01-real-project-matrix/scripts/exercise-real-projects-with-every-core-tool.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
node tools/support/compat/fixtures/tool-matrix-report.ts \
  --shard-index "$FIXTURE_SHARD_INDEX" \
  --shard-count "$FIXTURE_SHARD_COUNT" \
  --vize-bin target/ci/vize \
  --timeout-ms "$CORE_TOOLS_TIMEOUT_MS" \
  --output-dir "$FIXTURE_REPORT_DIR"
