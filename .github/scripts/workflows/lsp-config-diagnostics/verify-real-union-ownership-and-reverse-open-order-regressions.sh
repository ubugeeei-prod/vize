#!/usr/bin/env bash
cargo test --locked --profile ci -p vize_canon --features legacy --lib corsa_bridge::vue_document_package_tests::symlink_workspace::
cargo test --locked --profile ci -p vize_canon --features legacy --lib batch::virtual_project::materialize::alias_transition_tests::
cargo test --locked --profile ci -p vize_canon --features legacy --lib corsa_bridge::vue_dependencies_alias::context::cache::tests::
cargo clippy --locked --profile ci -p vize_canon --features legacy --lib -- -D warnings
