#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/actions/test-native-selected-template/scripts/capture-and-execute-original-selected-template-modules.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
rm -f "$VIZE_L4_SELECTED_TEMPLATE_CAPTURE"
cargo test -p vize_l4 --profile ci --test native_selected_template complete_original_selected_templates_keep_custody_and_exact_output -- --exact
vp node tools/support/compat/davinci/selected-template-dom.ts "$VIZE_L4_SELECTED_TEMPLATE_CAPTURE"
