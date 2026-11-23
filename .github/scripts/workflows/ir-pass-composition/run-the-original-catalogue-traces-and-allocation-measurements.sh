#!/usr/bin/env bash
set -euo pipefail
cargo clippy --locked --profile ci-opt -p vize_l0 --features streamed-pass-groups --lib --no-deps -- -D warnings
cargo test --locked --profile ci-opt -p vize_l0 --features streamed-pass-groups --test grouped_passes -- --include-ignored --nocapture --test-threads=1
cargo test --locked --profile ci-opt -p vize_l0 --features streamed-pass-groups --lib pass::
cargo test --locked --profile ci-opt -p vize_l3 --features streamed-pass-groups --test placement
