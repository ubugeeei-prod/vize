#!/usr/bin/env bash
vp exec node tools/support/compat/davinci/lsp-build-identity.ts target/x86_64-unknown-linux-gnu/release/vize release "${RUNNER_TEMP}/release-build.jsonl" "${RUNNER_TEMP}/release-build-identity.json"
