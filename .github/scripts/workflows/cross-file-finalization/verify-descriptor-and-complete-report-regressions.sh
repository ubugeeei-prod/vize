#!/usr/bin/env bash
mkdir -p target/vize-tests/cross-file-sfc-effects
cargo test --locked --profile ci -p vize --lib -- commands::lint::cross_file::effect_summary_tests:: --nocapture > target/vize-tests/cross-file-sfc-effects/rust.stdout 2> target/vize-tests/cross-file-sfc-effects/rust.stderr
cat target/vize-tests/cross-file-sfc-effects/rust.stdout
node "$VIZE_CI_FILES/verify-descriptor-and-complete-report-regressions.ts"
