#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/01-rust-target-seed/scripts/build-the-pr-feature-lanes.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
cargo test -p vize_armature --features native-lex-parity,legacy --lib --test markup_lex_parity --no-run
cargo test -p vize_l1 --features legacy-differential --test surface_corpus --no-run
