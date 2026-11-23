#!/usr/bin/env bash
set -o pipefail
cargo test --locked --profile ci -p vize_patina --lib linter::engine::batch::tests::component_imports_preserve_dynamic_module_namespace_packets -- --exact --nocapture 2>&1 | tee target/patina-native-boolean/dynamic-module-namespace.txt
