#!/usr/bin/env bash
cargo test --locked -p vize_croquis -p vize_croquis_cf --lib
cargo clippy --locked -p vize_croquis -p vize_croquis_cf --all-targets -- -D warnings
cargo check --locked -p vize_croquis -p vize_croquis_cf --target wasm32-unknown-unknown
