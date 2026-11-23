#!/usr/bin/env bash
set -euo pipefail
flags=(--locked --profile ci-opt -p vize_glyph --features retained-script)
cargo test "${flags[@]}" --lib script::snapshot::
cargo test "${flags[@]}" --test auto_line_endings --test sfc_empty_script_blocks --test import_sorting --test program_snapshot
git ls-files -z -- '*.vue' '*.vue.txt' '*.js' '*.mjs' '*.cjs' '*.jsx' '*.ts' '*.mts' '*.cts' '*.tsx' > "$VIZE_RETAINED_FORMAT_MANIFEST"
cargo test "${flags[@]}" --lib script::snapshot::tests::corpus::original_sources -- --exact --ignored --nocapture
cargo test "${flags[@]}" --lib script::snapshot::tests::profile::paired_formatter_requests -- --exact --ignored --nocapture
cargo test "${flags[@]}" --lib script::snapshot::tests::profile::corpus::original_formatter_requests -- --exact --ignored --nocapture
