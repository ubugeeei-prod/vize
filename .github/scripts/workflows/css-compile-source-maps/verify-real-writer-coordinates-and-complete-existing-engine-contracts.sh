#!/usr/bin/env bash
test "$(git rev-parse HEAD)" = "$VIZE_CSS_MAP_SOURCE_SHA"
cargo test --locked --profile ci -p vize_lightningcss --test vize_printer_utf16 --test vize_source_map_contract
cargo test --locked --profile ci -p vize_atelier_sfc --test css_source_maps --test css_v_bind_source_maps --test css_engine_panic_boundary --test css_nesting_guard
cargo test --locked --profile ci -p vize_croquis --test css_v_bind_origins
cargo test --locked --profile ci -p vize_atelier_sfc --lib css::
