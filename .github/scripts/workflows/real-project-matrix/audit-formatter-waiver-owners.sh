#!/usr/bin/env bash
node tools/support/compat/fixtures/glyph-corpus-waiver-audit.ts \
  --output "$FIXTURE_REPORT_DIR/glyph-waiver-issues.json"
