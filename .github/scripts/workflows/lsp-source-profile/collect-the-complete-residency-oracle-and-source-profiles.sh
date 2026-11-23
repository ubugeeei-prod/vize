#!/usr/bin/env bash
set -euo pipefail
for files in 100 500 2500; do
  vp exec node tools/support/compat/davinci/lsp-resources.ts \
    --server target/ci-opt/vize --files "$files" --warm-providers 16 --runs 1 --idle-seconds 10 \
    --build 'ci-opt with profiling allocator and source-operation counters; observation only' \
    --profile-json-dir "${RUNNER_TEMP}/source-${files}" \
    --out "${RUNNER_TEMP}/source-${files}.json"
done
