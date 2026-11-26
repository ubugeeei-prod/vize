#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/01-pr-rust/scripts/require-source-test-cases.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
test -s "$VIZE_TEST_CASES"
