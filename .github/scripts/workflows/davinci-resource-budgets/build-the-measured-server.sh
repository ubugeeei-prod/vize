#!/usr/bin/env bash
cargo build --profile ci-opt -p vize --config 'profile.ci-opt.inherits="release"' --config 'profile.ci-opt.lto="thin"' --config 'profile.ci-opt.codegen-units=16' --message-format=json-render-diagnostics > "${RUNNER_TEMP}/resource-build.jsonl"
