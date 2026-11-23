#!/usr/bin/env bash
set -euo pipefail
export VIZE_NATIVE_NAVIGATION_INPUTS="$RUNNER_TEMP/navigation-sources"
cargo test --locked --profile ci-opt -p vize_maestro --features experimental-navigation-products,micro-profiling --lib paired_original_navigation_requests -- --ignored --nocapture --test-threads=1
