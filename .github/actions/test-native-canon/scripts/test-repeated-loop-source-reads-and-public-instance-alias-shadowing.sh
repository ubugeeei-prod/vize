#!/usr/bin/env bash
unset VIZE_TEST_DISABLE_TSGO
cargo test --locked --profile ci -p vize_canon --lib batch::type_checker::tests::recent_issues::setup_custom_properties::loop_sources_keep_authored_reads_and_parent_scope -- --exact --nocapture
