#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/01-real-project-matrix/scripts/audit-formatter-waiver-owners.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
node tools/support/compat/fixtures/glyph-corpus-waiver-audit.ts \
  --output "$FIXTURE_REPORT_DIR/glyph-waiver-issues.json"
