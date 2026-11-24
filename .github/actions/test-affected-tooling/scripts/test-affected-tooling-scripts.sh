#!/usr/bin/env bash
node tools/ci/commands/verify-tool-layout.ts && node tools/support/compat/github/run-tooling-tests.mjs
