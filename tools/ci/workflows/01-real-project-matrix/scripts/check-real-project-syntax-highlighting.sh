#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/01-real-project-matrix/scripts/check-real-project-syntax-highlighting.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
node --test --test-concurrency=1 \
  tests/tooling/real-project-syntax-highlighting.test.ts
test -s "$FIXTURE_REPORT_DIR/syntax-highlighter-summary.json"
