#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/03-lsp-resources/scripts/measure-10000-open-buffers-16-production-providers-and-2.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
timeout \
  --signal=TERM \
  --kill-after=30s 45m vp exec node tools/support/compat/davinci/lsp-resources.ts \
  --server target/x86_64-unknown-linux-gnu/release/vize \
  --build "release: fat LTO, one codegen unit" \
  --files 10000 \
  --warm-providers 16 \
  --runs 1 \
  --idle-seconds 10 \
  --out "${RUNNER_TEMP}/release-lsp-resource.json"
