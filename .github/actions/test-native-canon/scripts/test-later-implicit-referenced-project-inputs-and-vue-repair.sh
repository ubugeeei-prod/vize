#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/actions/test-native-canon/scripts/test-later-implicit-referenced-project-inputs-and-vue-repair.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
unset VIZE_TEST_DISABLE_TSGO
cargo test --locked --profile ci -p vize --test integration -- check_tsconfig_mixed_project_graph_cli::default_reference::later_default_reference_checks_vue_template_and_repair --exact --nocapture
