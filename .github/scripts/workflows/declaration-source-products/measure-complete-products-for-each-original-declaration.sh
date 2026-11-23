#!/usr/bin/env bash
set -euo pipefail
export VIZE_DECLARATION_INPUTS="$RUNNER_TEMP/declaration-inputs"
cargo test --locked --profile ci-opt -p vize_maestro --features experimental-declaration-products,micro-profiling --lib paired_declaration_products -- --ignored --nocapture --test-threads=1
