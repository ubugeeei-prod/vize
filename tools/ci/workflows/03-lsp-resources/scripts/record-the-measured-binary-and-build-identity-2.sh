#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/03-lsp-resources/scripts/record-the-measured-binary-and-build-identity-2.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
vp exec node tools/support/compat/davinci/lsp-build-identity.ts target/x86_64-unknown-linux-gnu/release/vize release "${RUNNER_TEMP}/release-build.jsonl" "${RUNNER_TEMP}/release-build-identity.json"
