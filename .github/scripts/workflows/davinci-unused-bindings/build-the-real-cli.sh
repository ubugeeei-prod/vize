#!/usr/bin/env bash
set -o pipefail
cargo build --profile ci --message-format=json-render-diagnostics -p vize | tee target/unused-bindings-units/ci-vize.jsonl
