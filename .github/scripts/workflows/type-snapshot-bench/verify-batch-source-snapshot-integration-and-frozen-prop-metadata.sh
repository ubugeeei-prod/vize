#!/usr/bin/env bash
cargo test --locked -p vize_canon --lib batch::virtual_project::tests::type_source_snapshot
cargo test --locked -p vize_atelier_sfc --lib croquis::tests
cargo clippy --locked -p vize_canon --lib -- -D warnings
