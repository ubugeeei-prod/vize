#!/usr/bin/env bash
test "$(git rev-parse HEAD)" = "$SOURCE_SHA"
cargo clean --profile "$BUILD_PROFILE" -p vize
cargo build --locked --profile "$BUILD_PROFILE" -p vize --bin vize
