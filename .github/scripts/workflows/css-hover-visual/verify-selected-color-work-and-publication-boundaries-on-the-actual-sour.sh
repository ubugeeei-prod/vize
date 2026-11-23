#!/usr/bin/env bash
mkdir -p target/vize-tests/css-color-literal-hover
cargo test --locked --profile ci -p vize_maestro --lib server::annotations::document_color:: -- --nocapture
