#!/usr/bin/env bash
set -euo pipefail
unset VIZE_TEST_DISABLE_TSGO
cargo test --locked --profile ci -p vize --test integration -- check_options_api_inherited_members_cli:: --nocapture
