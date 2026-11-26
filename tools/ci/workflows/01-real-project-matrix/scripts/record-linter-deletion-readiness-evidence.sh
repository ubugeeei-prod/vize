#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/01-real-project-matrix/scripts/record-linter-deletion-readiness-evidence.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
node tests/differential/linter-deletion-recorder.ts "$FIXTURE_REPORT_DIR" && test -s "$FIXTURE_REPORT_DIR/linter-deletion-readiness.json"
