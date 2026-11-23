#!/usr/bin/env bash
cargo test --locked --profile ci -p vize_atelier_sfc native::setup::tests::eight_whole_setup_modules_and_maps_match_actual_source_bound_pinned_fixtures -- --exact
cargo test --locked --profile ci -p vize_atelier_sfc native::setup::tests::constants::five_whole_const_setup_modules_and_maps_match_actual_source_bound_pinned_fixtures -- --exact
cargo test --locked --profile ci -p vize_atelier_sfc native::setup::tests::typescript::three_whole_ts_primitive_setup_modules_and_maps_match_actual_source_bound_pinned_fixtures -- --exact
cargo test --locked --profile ci -p vize_atelier_sfc native::setup::tests::annotations::three_source_owned_annotation_modules_and_maps_are_captured -- --exact
cargo test --locked --profile ci -p vize_atelier_sfc native::ordinary::tests::three_whole_ordinary_empty_modules_and_maps_are_captured -- --exact
cargo test --locked --profile ci -p vize_atelier_sfc native::setup::tests::strict --lib
vp node --test tests/tooling/native-sfc-js-setup-reference.test.ts tests/tooling/native-sfc-primitive-setup-reference.test.ts tests/tooling/native-sfc-annotation-setup-reference.test.ts tests/tooling/native-sfc-strict-setup.test.ts tests/tooling/native-sfc-ordinary-empty-reference.test.ts
