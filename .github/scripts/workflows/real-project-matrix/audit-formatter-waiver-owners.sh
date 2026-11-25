#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/real-project-matrix/audit-formatter-waiver-owners.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
node tools/support/compat/fixtures/glyph-corpus-waiver-audit.ts \
  --output "$FIXTURE_REPORT_DIR/glyph-waiver-issues.json"
