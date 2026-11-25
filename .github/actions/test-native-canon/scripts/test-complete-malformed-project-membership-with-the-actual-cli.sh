#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/actions/test-native-canon/scripts/test-complete-malformed-project-membership-with-the-actual-cli.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
unset VIZE_TEST_DISABLE_TSGO
cargo test --locked --profile ci -p vize_canon --test empty_vue_star_exports -- configured_native_refuses_erasure_at_unproved_source_boundaries --exact --nocapture
