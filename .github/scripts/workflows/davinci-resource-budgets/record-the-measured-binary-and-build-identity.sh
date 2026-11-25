#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/davinci-resource-budgets/record-the-measured-binary-and-build-identity.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
vp exec node tools/support/compat/davinci/lsp-build-identity.ts target/ci-opt/vize ci-opt "${RUNNER_TEMP}/resource-build.jsonl" "${RUNNER_TEMP}/resource-build-identity.json"
