#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/01-real-project-matrix/scripts/check-real-project-lsp-lifecycle.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
node tests/differential/build-receipt.ts
node tests/tooling/support/lsp/verify-session-captures.ts snapshot "$FIXTURE_REPORT_DIR/lsp-wire-before.json"
printf '%s\n' 'Lifecycle timings include passive raw RPC recording; they are not paired performance acceptance.' > "$FIXTURE_REPORT_DIR/lsp-wire-timing-policy.txt"
if [ ! -x "$CORSA_PATH" ]; then
  echo "::error title=Missing typecheck runtime::vp install produced no executable at $CORSA_PATH"
  exit 1
fi
node --test --test-concurrency=1 \
  tests/tooling/real-project-lsp.test.ts
test -s "$FIXTURE_REPORT_DIR/lsp-lifecycle-summary.json"
