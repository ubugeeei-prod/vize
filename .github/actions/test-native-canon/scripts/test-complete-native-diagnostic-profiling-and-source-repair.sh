#!/usr/bin/env bash
set -euo pipefail
unset VIZE_TEST_DISABLE_TSGO
cargo test --locked --profile ci -p vize --test integration -- check_message_profile_cli::real_check_message_profile_preserves_complete_native_packets_and_repair --exact --nocapture
