#!/usr/bin/env bash
cargo test --locked --profile ci -p vize_atelier_sfc native_selected::tests::local::ten_original_block_local_components_and_maps_are_captured -- --exact
vp node --test tests/tooling/native-handler-local-sfc-dom-reference.test.ts
test -s "$VIZE_NATIVE_HANDLER_LOCAL_SFC_DOM_CAPTURE"
test -s "$VIZE_NATIVE_HANDLER_LOCAL_SFC_DOM_RUNTIME_CAPTURE"
