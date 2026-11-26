#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/01-check/scripts/verify-source-coverage-summary.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
test -s target/llvm-cov/source-summary.json
