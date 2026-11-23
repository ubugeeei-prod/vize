#!/usr/bin/env bash
cargo test --locked -p vize_atelier_sfc script::context::type_world
cargo test --locked -p vize_atelier_sfc script::context::source_snapshot
cargo clippy --locked -p vize_atelier_sfc --lib --example type_snapshot_bench -- -D warnings
