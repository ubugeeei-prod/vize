#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/actions/test-native-vue2-text-document/scripts/join-fresh-original-textview-outputs-to-pinned-whole-goldens.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
rm -f "$VIZE_GLYPH_VUE2_TEXT_CAPTURE"
cargo test --locked --profile ci -p vize_glyph --test native_vue2_text_document whole_print_goldens_and_idempotence_capture_only_real_original_docs -- --exact
test -s "$VIZE_GLYPH_VUE2_TEXT_CAPTURE"
vp node --test tests/tooling/native-vue2-text-document-oracle.test.ts
