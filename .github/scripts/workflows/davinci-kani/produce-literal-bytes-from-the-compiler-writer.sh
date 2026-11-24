#!/usr/bin/env bash
cargo test --locked --profile ci-opt -p vize_l0 --lib js_string::tests
cargo test --locked --profile ci-opt -p vize_l0 --test emitted_string_literals
