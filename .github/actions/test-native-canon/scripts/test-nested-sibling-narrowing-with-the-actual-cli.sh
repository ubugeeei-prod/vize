#!/usr/bin/env bash
set -euo pipefail
unset VIZE_TEST_DISABLE_TSGO
cargo test --locked --profile ci -p vize --test integration -- check_flow_depth_isolation_cli:: --nocapture
