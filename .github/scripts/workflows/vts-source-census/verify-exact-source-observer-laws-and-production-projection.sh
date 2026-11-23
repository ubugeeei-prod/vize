#!/usr/bin/env bash
set -euo pipefail
driver_sha=$GITHUB_WORKFLOW_SHA
if ! [[ "$driver_sha" =~ ^[0-9a-f]{40}$ ]]; then
  echo "::error title=VTS census driver drift::workflow_sha=$driver_sha"
  exit 1
fi
if ! git cat-file -e "$driver_sha^{commit}" 2>/dev/null; then
  git fetch --no-tags --depth=1 origin "$driver_sha"
fi
git cat-file -e "$driver_sha:.github/workflows/vts-source-census.yml"
actual_source=$(git rev-parse HEAD)
actual_parent=$(git rev-parse HEAD^)
if test "$actual_source" != "$SOURCE_SHA"; then
  echo "::error title=VTS census source drift::checkout=$actual_source requested=$SOURCE_SHA"
  exit 1
fi
if test "$actual_parent" != "$SOURCE_PARENT"; then
  echo "::error title=VTS census source parent mismatch::head_parent=$actual_parent requested_parent=$SOURCE_PARENT"
  exit 1
fi
node --test tests/tooling/vts-source-census.test.ts
node tools/benchmarks/scripts/vts-source-census-preflight.ts "$SOURCE_PARENT" "$SOURCE_SHA" "$driver_sha"
