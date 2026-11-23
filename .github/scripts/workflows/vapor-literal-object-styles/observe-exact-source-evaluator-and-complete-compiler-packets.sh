#!/usr/bin/env bash
set -euo pipefail
mkdir -p "$RUNNER_TEMP/vapor-literal-styles"
test "$(git rev-parse HEAD)" = "$QUALIFICATION_HEAD_SHA"
git diff --exit-code
node --test tests/tooling/vapor-style-api-fixtures.test.ts tests/tooling/vapor-style-authored-transition.test.ts tests/tooling/vapor-ssr-expose-successor.test.ts tests/tooling/vapor-style-successor-composition.test.ts
git rev-parse HEAD HEAD^{tree} > "$RUNNER_TEMP/vapor-literal-styles/head-source.txt"
cargo test --locked --profile ci -p vize_atelier_jsx --test literal_object_styles --test literal_style_source --test literal_style_sfc --test literal_style_maps -- --nocapture --test-threads=1
