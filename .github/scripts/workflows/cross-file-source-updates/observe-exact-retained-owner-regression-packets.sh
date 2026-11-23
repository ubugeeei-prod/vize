#!/usr/bin/env bash
mkdir -p target/vize-tests/cross-file-source-updates
test "$(git rev-parse HEAD)" = "$EXPECTED_SOURCE_SHA"
git diff --exit-code
git rev-parse HEAD HEAD:crates/vize_croquis_cf/src > target/vize-tests/cross-file-source-updates/source-identity.txt
rustc --version --verbose > target/vize-tests/cross-file-source-updates/rustc.txt
sha256sum Cargo.lock crates/vize_croquis_cf/src/analyzer/core/deps.rs crates/vize_croquis_cf/src/analyzer/core/deps/source_update_tests.rs > target/vize-tests/cross-file-source-updates/source-inputs.sha256
cp crates/vize_croquis_cf/src/analyzer/core/deps/source_update_tests.rs target/vize-tests/cross-file-source-updates/
cargo test --locked --profile ci -p vize_croquis_cf --lib -- analyzer::core::deps::source_update_tests:: --nocapture > target/vize-tests/cross-file-source-updates/rust.stdout 2> target/vize-tests/cross-file-source-updates/rust.stderr
cat target/vize-tests/cross-file-source-updates/rust.stdout
node "$VIZE_CI_FILES/observe-exact-retained-owner-regression-packets.ts"
