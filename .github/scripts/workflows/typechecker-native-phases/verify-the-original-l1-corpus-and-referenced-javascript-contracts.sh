#!/usr/bin/env bash
set -euo pipefail
cd "$NATIVE_PHASE_SOURCE_ROOT"
unset VIZE_TEST_DISABLE_TSGO
cargo test --locked --profile ci -p vize_l1_to_l2 --features legacy-differential --test davinci_lowering_corpus -- --nocapture
cargo test --locked --profile ci -p vize_l1_to_l2 --features legacy-differential --test davinci_dom_corpus -- --nocapture
cargo test --locked --profile ci -p vize_l1_to_l2 --features legacy-differential --test davinci_remarks_corpus -- --nocapture
VIZE_TEST_REQUIRE_TSGO=1 cargo test --locked --profile ci -p vize --test integration -- check_allowjs_imports_cli::referenced_projects_apply_allowjs_to_explicit_and_default_checks --nocapture
