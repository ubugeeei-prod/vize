#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/davinci-resource-budgets/verify-the-protocol-and-controlled-edit-oracle-2.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
vp exec node tools/support/compat/davinci/lsp-resources.ts \
  --server target/x86_64-unknown-linux-gnu/release/vize \
  --build "release: fat LTO, one codegen unit" \
  --files 2 \
  --runs 1 \
  --idle-seconds 10 \
  --out "${RUNNER_TEMP}/release-lsp-resource-smoke.json"
