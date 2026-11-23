#!/usr/bin/env bash
set -euo pipefail
cargo test --locked --profile ci-opt -p vize_l1 --features script-queries --lib container::vue::programs::
cargo test --locked --profile ci-opt -p vize_l2 --features script-queries --lib query::vue::
flags=(--locked --profile ci-opt -p vize_glyph --features retained-script,vize_l2/script-queries --test selected_script_programs)
git ls-files -z -- '*.vue' '*.vue.txt' > "$VIZE_SCRIPT_PROGRAMS_MANIFEST"
cargo test "${flags[@]}"
cargo test "${flags[@]}" original_sources -- --exact --ignored --nocapture
cargo test "${flags[@]}" profile::shared_program_requests -- --exact --ignored --nocapture
cargo test "${flags[@]}" profile::corpus::original_program_requests -- --exact --ignored --nocapture
