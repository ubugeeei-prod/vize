#!/usr/bin/env bash
if node "$GITHUB_WORKSPACE/tools/support/release/jsr/package.ts" is-public "${JSR_VERSION#v}"; then
  exit 0
fi
set +e
timeout --signal=INT --kill-after=30s 20m node "$GITHUB_WORKSPACE/tools/support/release/jsr/publish-with-fresh-provenance.ts" -- deno publish --config jsr.json
status=$?
set -e
if [[ $status -ne 0 && $status -ne 124 ]]; then
  exit "$status"
fi
