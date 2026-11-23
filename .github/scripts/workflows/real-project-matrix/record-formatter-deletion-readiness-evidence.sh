#!/usr/bin/env bash
node tests/differential/formatter-deletion-verifier.mjs "$FIXTURE_REPORT_DIR"
test -s "$FIXTURE_REPORT_DIR/formatter-deletion-readiness.json"
