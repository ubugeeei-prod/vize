#!/usr/bin/env bash
cargo test --locked --workspace --no-run
mkdir -p target/ci-walltime
cargo nextest list --locked --workspace --profile full --message-format json > target/ci-walltime/rust-selected-tests.json
