#!/usr/bin/env bash
cargo test -p vize_armature --features native-lex-parity,legacy --lib --test markup_lex_parity --no-run
cargo test -p vize_l1 --features legacy-differential --test surface_corpus --no-run
