#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/06-browser-oracles/scripts/first-newline-build-the-actual-cli-and-run-source-bound-browser-observations.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
git rev-parse HEAD > "$RUNNER_TEMP/first-newline-source.txt"
cargo build --locked --profile ci -p vize
cargo test --locked --profile ci -p vize_armature --lib
cargo test --locked --profile ci -p vize_l1_to_l2 --test integration -- pre_textarea_first_newline:: text_pass:: text_singleton::
vp node --test tests/tooling/pre-textarea-first-newline.test.mjs
mkdir -p "$RUNNER_TEMP/first-newline-tokenizer-current"
VIZE_TOKENIZER_CAPTURE_DIR="$RUNNER_TEMP/first-newline-tokenizer-current" cargo test --locked --profile ci -p vize_armature --features legacy --lib
