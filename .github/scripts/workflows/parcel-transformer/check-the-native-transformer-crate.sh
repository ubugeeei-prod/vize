#!/usr/bin/env bash
cargo test -p vize_parcel_transformer
cargo clippy -p vize_parcel_transformer --all-targets -- -D warnings
