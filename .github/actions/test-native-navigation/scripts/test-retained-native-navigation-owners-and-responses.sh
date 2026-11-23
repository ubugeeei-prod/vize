#!/usr/bin/env bash
cargo test --locked -p vize_maestro --features experimental-source-navigation --lib source_project:: -- --nocapture
cargo test --locked -p vize_maestro --features experimental-source-navigation --lib server::native_navigation:: -- --nocapture
cargo test --locked -p vize_maestro --features experimental-source-navigation --lib server::project_contexts:: -- --nocapture
cargo test --locked -p vize_maestro --features experimental-source-navigation --lib module_link -- --nocapture
cargo test --locked -p vize_maestro --no-default-features --features experimental-source-navigation --lib module_link -- --nocapture
cargo check --locked -p vize_maestro --no-default-features --features experimental-source-navigation
cargo clippy --locked -p vize_maestro --features experimental-source-navigation --all-targets -- -D warnings
