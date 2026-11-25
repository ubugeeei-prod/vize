#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/level-memory-bench/validate-exact-dispatch-and-pair-before-build.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
[[ "$MEMORY_BASE_SHA" =~ ^[0-9a-f]{40}$ ]]
[[ "$MEMORY_HEAD_SHA" =~ ^[0-9a-f]{40}$ ]]
[[ "$MEMORY_CONFLICTING_MODES" == false ]]
[[ "$MEMORY_BASE_SHA" != "$MEMORY_HEAD_SHA" ]]
[[ "$MEMORY_HEAD_SHA" == "$GITHUB_SHA" ]]
[[ "$(git -C base rev-parse HEAD)" == "$MEMORY_BASE_SHA" ]]
[[ "$(git -C head rev-parse HEAD)" == "$MEMORY_HEAD_SHA" ]]
git -C head merge-base --is-ancestor "$MEMORY_BASE_SHA" "$MEMORY_HEAD_SHA"
