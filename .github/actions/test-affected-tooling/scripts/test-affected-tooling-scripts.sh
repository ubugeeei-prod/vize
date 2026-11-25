#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/actions/test-affected-tooling/scripts/test-affected-tooling-scripts.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
node tools/commands/ci/verify-tool-layout.ts && node tools/support/compat/github/run-tooling-tests.mjs
