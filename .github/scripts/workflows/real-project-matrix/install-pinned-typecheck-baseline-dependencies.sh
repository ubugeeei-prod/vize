#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/real-project-matrix/install-pinned-typecheck-baseline-dependencies.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
node --experimental-strip-types tools/commands/fixtures/typecheck-dependency-prepare.ts \
  --output-dir "$FIXTURE_REPORT_DIR" \
  --shard-index "$FIXTURE_SHARD_INDEX" \
  --shard-count "$FIXTURE_SHARD_COUNT" \
  --timeout-ms 600000
