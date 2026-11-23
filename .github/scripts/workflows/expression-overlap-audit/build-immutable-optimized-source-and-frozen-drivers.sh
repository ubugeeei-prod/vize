#!/usr/bin/env bash
set -euo pipefail
cargo +nightly-2026-10-23 build --locked --profile ci-opt -p vize_oxc_parser --example failed_speculation_profile -p vize_atelier_core --example fuzz_expression_10401_profile --example fuzz_expression_raw_close_observe 2>&1 | tee ../evidence/optimized-build.log
