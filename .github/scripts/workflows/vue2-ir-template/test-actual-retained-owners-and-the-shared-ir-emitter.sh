#!/usr/bin/env bash
set -euo pipefail
cargo test --locked --profile ci -p vize_l1_to_l2 --lib native_vue2:: -- --nocapture 2>&1 | tee target/vize-tests/vue2-ir/lower.log
cargo test --locked --profile ci -p vize_l4 --lib targets::vue2:: -- --nocapture 2>&1 | tee target/vize-tests/vue2-ir/emit.log
