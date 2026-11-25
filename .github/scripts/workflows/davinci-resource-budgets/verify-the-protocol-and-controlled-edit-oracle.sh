#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/davinci-resource-budgets/verify-the-protocol-and-controlled-edit-oracle.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
vp exec node tools/support/compat/davinci/lsp-resources.ts \
  --server target/ci-opt/vize \
  --files 2 \
  --runs 1 \
  --idle-seconds 10 \
  --build "ci-opt: release, thin LTO, 16 codegen units" \
  --out "${RUNNER_TEMP}/lsp-resource-smoke.json"
