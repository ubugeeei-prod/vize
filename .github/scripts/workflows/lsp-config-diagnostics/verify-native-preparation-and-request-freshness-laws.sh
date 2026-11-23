#!/usr/bin/env bash
cargo test --locked --profile ci -p vize_maestro --features legacy --lib server::initial_diagnostics::
cargo test --locked --profile ci -p vize_maestro --features legacy --lib server::native_requests::
cargo test --locked --profile ci -p vize_maestro --features legacy --lib server::state::project_contexts:: -- --nocapture
cargo clippy --locked --profile ci -p vize_maestro --features legacy --all-targets -- -D warnings
