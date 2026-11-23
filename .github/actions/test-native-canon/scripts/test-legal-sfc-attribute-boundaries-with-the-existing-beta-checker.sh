#!/usr/bin/env bash
set -euo pipefail
unset VIZE_TEST_DISABLE_TSGO
capture="$VIZE_RECOVERED_ATTRIBUTE_CAPTURE_DIR"
mkdir -p "$capture"
set +e
cargo test --locked --profile ci -p vize --test integration -- content_mapper_tsgo_cli::recovered_attribute_boundary_11731::legal_quoted_boundaries_keep_neighbor_and_unused_diagnostics --exact --nocapture --color never >"$capture/test.stdout" 2>"$capture/test.stderr"
status=$?
set -e
printf '%s\n' "$status" >"$capture/test.status"
cat "$capture/test.stdout"
cat "$capture/test.stderr" >&2
node "$GITHUB_ACTION_PATH/scripts/test-legal-sfc-attribute-boundaries-with-the-existing-beta-checker.ts" "$capture"
