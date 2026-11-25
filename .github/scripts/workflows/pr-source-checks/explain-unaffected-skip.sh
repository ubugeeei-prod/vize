#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/pr-source-checks/explain-unaffected-skip.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
echo 'The WASM release crate depends on no changed Rust package.' >> "$GITHUB_STEP_SUMMARY"
