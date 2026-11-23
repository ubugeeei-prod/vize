#!/usr/bin/env bash
cargo clippy --locked --profile ci -p vize_fresco --features napi --lib -- -D warnings
cargo clippy --locked --profile ci -p vize_fresco --lib --tests -- -D warnings
