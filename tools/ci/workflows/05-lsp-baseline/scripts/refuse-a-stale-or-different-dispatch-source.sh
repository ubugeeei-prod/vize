#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/05-lsp-baseline/scripts/refuse-a-stale-or-different-dispatch-source.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
[[ "$VIZE_BASELINE_EXPECTED_SOURCE" =~ ^[0-9a-f]{40}$ ]]
test "$(git rev-parse HEAD)" = "$VIZE_BASELINE_EXPECTED_SOURCE"
test "$GITHUB_SHA" = "$VIZE_BASELINE_EXPECTED_SOURCE"
