#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/level-html-content-model/scoped-nesting-compile-current-source-and-retain-all-prior-scoped-css-oracles.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
test "$(git rev-parse HEAD)" = "$HEAD_SHA"
cargo test --locked --profile ci -p vize_atelier_sfc --lib vite_plugin::css_scope:: -- --nocapture
cargo test --locked --profile ci -p vize_atelier_sfc --test scoped_nesting_specificity -- --nocapture
cargo test --locked --profile ci -p vize_atelier_sfc --test mixed_style_scope -- --nocapture
