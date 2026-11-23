#!/usr/bin/env bash
cargo build --profile ci-opt -p vize --features profiling --message-format=json-render-diagnostics > "${RUNNER_TEMP}/source-build.jsonl"
