#!/usr/bin/env bash
cargo test --profile ci -p vize_croquis_cf
cargo clippy -p vize_croquis_cf --lib --tests -- -D warnings
