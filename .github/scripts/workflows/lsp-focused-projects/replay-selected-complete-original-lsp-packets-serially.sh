#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/lsp-focused-projects/replay-selected-complete-original-lsp-packets-serially.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
test -x "$CORSA_PATH"
node tests/tooling/support/lsp/verify-session-captures.ts snapshot "$CAMPAIGN_DIR/lsp-wire-before.json"
node tools/support/compat/github/lsp-focused-lifecycle.ts
