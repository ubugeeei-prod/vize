#!/usr/bin/env bash
cargo test --locked -p vize_doctor filter
cargo clippy --locked -p vize_doctor --all-targets -- -D warnings
