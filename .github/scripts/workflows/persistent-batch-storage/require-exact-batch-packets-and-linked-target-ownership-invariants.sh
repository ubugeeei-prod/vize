#!/usr/bin/env bash
cargo nextest run -p vize_canon --lib --test fix_history_diagnostics --cargo-profile ci --profile full -E 'test(relative_sibling_imports_preserve_complete_package_diagnostics) | test(persistent_namespace) | test(installed_sources) | test(replacing_a_hardlinked_cache_leaf)' --no-tests=fail
cp target/nextest/full/junit.xml target/persistent-storage-evidence/canon.junit.xml
