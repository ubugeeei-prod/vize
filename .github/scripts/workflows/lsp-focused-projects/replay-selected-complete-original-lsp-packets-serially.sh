#!/usr/bin/env bash
test -x "$CORSA_PATH"
node tests/tooling/support/lsp/verify-session-captures.ts snapshot "$CAMPAIGN_DIR/lsp-wire-before.json"
node tools/support/compat/github/lsp-focused-lifecycle.ts
