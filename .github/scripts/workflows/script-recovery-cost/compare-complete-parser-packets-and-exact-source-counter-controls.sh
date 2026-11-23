#!/usr/bin/env bash
set -euo pipefail
cargo +1.100.0 test --locked --profile ci -p vize_croquis script_parser::recovery 2>&1 | tee evidence/recovery-packets.log
cargo +1.100.0 test --locked --profile ci -p vize_l0 --test source_operations_profile --features micro-profiling 2>&1 | tee evidence/callsite-controls.log
