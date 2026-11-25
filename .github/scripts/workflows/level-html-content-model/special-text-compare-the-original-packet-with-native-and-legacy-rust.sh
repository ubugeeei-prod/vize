#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/l1-document-special-text/compare-the-original-packet-with-native-and-legacy-rust.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
test "$(git rev-parse HEAD)" = "$HEAD_SHA"
cargo test --locked --profile ci -p vize_l1 \
  --test html_body_elements \
  --test html_body_elements_legacy_oracle \
  --test html_disclosure_elements \
  --test html_disclosure_elements_legacy_oracle \
  --test html_embedded_fallback \
  --test html_embedded_fallback_legacy_oracle \
  --test html_foreign_content \
  --test html_foreign_content_legacy_oracle \
  --test html_phrasing \
  --test html_phrasing_legacy_oracle \
  --test html_select \
  --test html_select_legacy_oracle \
  --test html_special_text \
  --test html_special_text_legacy_oracle
