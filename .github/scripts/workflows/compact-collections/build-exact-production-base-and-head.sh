#!/usr/bin/env bash
(cd base && CARGO_TARGET_DIR=../head/target/base cargo build --release -p vize_croquis_cf --example compact_collections)
(cd head && CARGO_TARGET_DIR=target/head cargo build --release --locked -p vize_croquis_cf --example compact_collections)
