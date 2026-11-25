#!/usr/bin/env bash
git rev-parse HEAD > "$RUNNER_TEMP/first-newline-source.txt"
cargo build --locked --profile ci -p vize
cargo test --locked --profile ci -p vize_armature --lib
cargo test --locked --profile ci -p vize_l1_to_l2 --test integration -- pre_textarea_first_newline:: text_pass:: text_singleton::
vp node --test tests/tooling/pre-textarea-first-newline.test.mjs
mkdir -p "$RUNNER_TEMP/first-newline-tokenizer-current"
VIZE_TOKENIZER_CAPTURE_DIR="$RUNNER_TEMP/first-newline-tokenizer-current" cargo test --locked --profile ci -p vize_armature --features legacy --lib
