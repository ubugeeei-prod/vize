#!/usr/bin/env bash
cargo build --locked --manifest-path target/finalization-order-base/Cargo.toml --profile ci -p vize --example cross_file_finalization_order --target-dir target/finalization-order-base-build
cargo build --locked --profile ci -p vize --example cross_file_finalization_order --target-dir target/finalization-order-head-build
