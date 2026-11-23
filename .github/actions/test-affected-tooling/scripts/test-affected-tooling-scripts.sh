#!/usr/bin/env bash
node tools/commands/ci/verify-tool-layout.ts && node tools/support/compat/github/run-tooling-tests.mjs
