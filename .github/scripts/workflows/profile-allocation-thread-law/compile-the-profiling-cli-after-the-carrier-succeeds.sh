#!/usr/bin/env bash
set -euo pipefail
cargo +1.100.0 build --locked --profile ci -p vize --features profiling --message-format json > evidence/profiling-cli-artifacts.jsonl
