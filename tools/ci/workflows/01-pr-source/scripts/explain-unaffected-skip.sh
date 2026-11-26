#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/01-pr-source/scripts/explain-unaffected-skip.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
echo 'The WASM release crate depends on no changed Rust package.' >> "$GITHUB_STEP_SUMMARY"
