#!/usr/bin/env bash
cargo clippy --workspace --profile ci -- -D warnings -D clippy::wildcard_imports
cargo test --workspace --profile ci --no-run
