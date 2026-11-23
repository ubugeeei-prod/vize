#!/usr/bin/env bash
cargo build --locked --manifest-path target/finalization-compare-base/Cargo.toml --profile ci-opt -p vize --target-dir target/finalization-compare-base-build
cargo build --locked --profile ci-opt -p vize --target-dir target/finalization-compare-head-build
