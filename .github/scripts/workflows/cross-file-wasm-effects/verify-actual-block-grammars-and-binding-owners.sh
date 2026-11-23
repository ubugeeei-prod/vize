#!/usr/bin/env bash
mkdir -p target/vize-tests/cross-file-wasm-effects
cargo test --locked --profile ci -p vize_vitrine --no-default-features --features wasm --lib -- wasm::cross_file::effect_summary::tests:: --nocapture > target/vize-tests/cross-file-wasm-effects/rust.stdout 2> target/vize-tests/cross-file-wasm-effects/rust.stderr
cat target/vize-tests/cross-file-wasm-effects/rust.stdout
node "$VIZE_CI_FILES/verify-actual-block-grammars-and-binding-owners.ts"
