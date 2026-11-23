#!/usr/bin/env bash
rm -f "$VIZE_GLYPH_VUE2_TEXT_CAPTURE"
cargo test --locked --profile ci -p vize_glyph --test native_vue2_text_document whole_print_goldens_and_idempotence_capture_only_real_original_docs -- --exact
test -s "$VIZE_GLYPH_VUE2_TEXT_CAPTURE"
vp node --test tests/tooling/native-vue2-text-document-oracle.test.ts
