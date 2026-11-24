#!/usr/bin/env bash
cargo test --locked -p davinci_harness --lib
cargo test --locked -p davinci_harness --lib --features instruction-counts
