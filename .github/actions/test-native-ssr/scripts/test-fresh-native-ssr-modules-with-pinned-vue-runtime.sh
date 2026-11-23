#!/usr/bin/env bash
cargo test --locked --profile ci -p vize_l4 --test native_ssr --test native_ssr_roles --test native_selected_ssr -- --nocapture
cargo test --locked --profile ci -p vize_atelier_jsx --test ssr -- --nocapture
cargo test --locked --profile ci -p vize_atelier_jsx --test falsy_and actual_values_updates_and_hydration_match_jsx_child_semantics -- --exact --nocapture
vp node --test tests/tooling/l4-native-ssr-reference.test.ts tests/tooling/l4-selected-ssr-reference.test.ts tests/tooling/native-ssr-static-class-reference.test.ts tests/tooling/jsx-runtime-slots-ssr.test.ts
NODE_ENV=production vp node --test tests/tooling/native-ssr-static-class-reference.test.ts
