#!/usr/bin/env bash
set -euo pipefail
node tools/benchmarks/scripts/typechecker-cache-source.ts --source-root "$GITHUB_WORKSPACE" --target-dir "$RUNNER_TEMP/cache-source-target" --receipt "$RUNNER_TEMP/cache-evidence/source-build.json"
