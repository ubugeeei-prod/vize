#!/usr/bin/env bash
set -euo pipefail
cargo test --locked --profile ci -p vize_maestro --lib server::annotations::document_color:: -- --nocapture 2>&1 | tee target/vize-tests/css-color-literal-hover/native.log
