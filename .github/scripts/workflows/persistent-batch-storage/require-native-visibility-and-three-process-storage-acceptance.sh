#!/usr/bin/env bash
cargo nextest run -p vize --test integration --cargo-profile ci --profile full -E 'test(relative_sibling_sfc_keeps_package_storage_and_exact_repeated_diagnostics) | test(type_config_anchor_does_not_expose_app_modules_to_sibling_sources) | test(expanded_program_cache_redirect_stays_outside_authored_workspace)' --no-tests=fail
cp target/nextest/full/junit.xml target/persistent-storage-evidence/cli.junit.xml
sha256sum target/ci/vize > target/persistent-storage-evidence/cli.sha256
tar -czf target/persistent-storage-evidence/source-cli-linux-x64.tgz -C target/ci vize
