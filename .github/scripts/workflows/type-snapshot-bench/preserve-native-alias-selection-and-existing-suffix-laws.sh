#!/usr/bin/env bash
cargo test --locked -p vize_canon --lib precedence_tests
cargo test --locked -p vize_canon --lib dependency_scan::resolution::alias_tests
cargo test --locked -p vize_canon --lib batch::virtual_project::tests::alias_rewrite
cargo test --locked -p vize_canon --lib app_only_suffix_alias_reports_and_repairs_the_authored_child
cargo test --locked -p vize --lib commands::check::imports_aliases::tests::selection -- --nocapture
node --test tools/benchmarks/scripts/canon-alias-collector.test.mjs
