#!/usr/bin/env bash
node tools/support/github/build-napi-package.ts npm/native ../../crates/vize_vitrine/Cargo.toml vize_vitrine x86_64-unknown-linux-gnu --cross-compile
cp npm/native/vize-vitrine.linux-x64-gnu.node "$RUNNER_TEMP/floor/vize-vitrine.node"
