#!/usr/bin/env bash
cargo test --locked --profile ci -p vize_atelier_core --lib inline_immutable_setup_defaults_keep_exact_authored_parameter_packets -- --nocapture
cargo test --locked --profile ci -p vize_l1_to_l2 --lib inline_immutable_setup_defaults_keep_exact_authored_parameter_packets -- --nocapture
cargo test --locked --profile ci -p vize_atelier_sfc --features legacy-dom-differential --test slot_default_setup_bindings_8142 -- --nocapture
