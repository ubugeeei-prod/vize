#!/usr/bin/env bash
set -euo pipefail

export RUSTFLAGS="-D warnings"

cargo check -p vize_maestro --no-default-features
cargo test -p vize_maestro --no-default-features --test non_native_structural
cargo check -p vize_maestro --no-default-features --features glyph
cargo test -p vize_maestro --no-default-features --features glyph --test non_native_structural
