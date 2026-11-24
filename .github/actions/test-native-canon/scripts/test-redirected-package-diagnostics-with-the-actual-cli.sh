#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/actions/test-native-canon/scripts/test-redirected-package-diagnostics-with-the-actual-cli.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
unset VIZE_TEST_DISABLE_TSGO
cargo test --locked --profile ci -p vize --test integration -- check_package_import_redirects_cli:: --nocapture
