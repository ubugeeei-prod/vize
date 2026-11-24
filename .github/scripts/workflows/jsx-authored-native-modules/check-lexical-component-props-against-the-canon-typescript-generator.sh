#!/usr/bin/env bash
cargo test --locked --profile ci -p vize_canon --features component-projection-tests --test component_prop_projection
cargo run --locked --profile ci -p vize_canon --features component-projection-tests --example component_projection_alloc
cargo test --locked --profile ci -p vize_canon --features component-projection-tests --test component_prop_projection -- corpus::component_props_in_original_modules --exact --ignored --nocapture
