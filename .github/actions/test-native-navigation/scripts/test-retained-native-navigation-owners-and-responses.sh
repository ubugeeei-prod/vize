#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/actions/test-native-navigation/scripts/test-retained-native-navigation-owners-and-responses.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
cargo test --locked -p vize_maestro --features experimental-source-navigation --lib source_project:: -- --nocapture
cargo test --locked -p vize_maestro --features experimental-source-navigation --lib server::native_navigation:: -- --nocapture
cargo test --locked -p vize_maestro --features experimental-source-navigation --lib server::project_contexts:: -- --nocapture
cargo test --locked -p vize_maestro --features experimental-source-navigation --lib module_link -- --nocapture
cargo test --locked -p vize_maestro --no-default-features --features experimental-source-navigation --lib module_link -- --nocapture
cargo check --locked -p vize_maestro --no-default-features --features experimental-source-navigation
cargo clippy --locked -p vize_maestro --features experimental-source-navigation --all-targets -- -D warnings
