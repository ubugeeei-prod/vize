#!/usr/bin/env bash
set -euo pipefail
cargo test --locked --profile ci -p vize_maestro --lib source_project::navigation::intrinsic_tests:: -- --nocapture 2>&1 | tee target/vize-tests/jsx-intrinsic-hover/native.log
