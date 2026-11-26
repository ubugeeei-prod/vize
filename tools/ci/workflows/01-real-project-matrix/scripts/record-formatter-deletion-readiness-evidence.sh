#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/01-real-project-matrix/scripts/record-formatter-deletion-readiness-evidence.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
node tests/differential/formatter-deletion-verifier.mjs "$FIXTURE_REPORT_DIR"
test -s "$FIXTURE_REPORT_DIR/formatter-deletion-readiness.json"
