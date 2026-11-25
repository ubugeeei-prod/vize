#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/real-project-matrix/record-formatter-deletion-readiness-evidence.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
node tests/differential/formatter-deletion-verifier.mjs "$FIXTURE_REPORT_DIR"
test -s "$FIXTURE_REPORT_DIR/formatter-deletion-readiness.json"
