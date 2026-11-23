#!/usr/bin/env bash
mkdir -p "$VIZE10903_PROOF"
cargo test -p vize_atelier_sfc --lib compile::bindings::tests::normal_script_imports_preserve_vue_ref_boundaries -- --exact 2>&1 | tee "$VIZE10903_PROOF/classification-test.log"
node "$VIZE_CI_FILES/verify-exact-import-classifications-and-merge-flags.cjs"
