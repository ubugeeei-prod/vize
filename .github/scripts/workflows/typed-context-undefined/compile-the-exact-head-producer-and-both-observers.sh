#!/usr/bin/env bash
set -euo pipefail
directory="$RUNNER_TEMP/typed-context-undefined"
cargo test --locked --profile ci -p vize --test integration --no-run --message-format=json > "$directory/after-build.jsonl" 2> "$directory/after-build.stderr"
cargo test --locked --profile ci -p vize_canon --lib --no-run --message-format=json >> "$directory/after-build.jsonl" 2>> "$directory/after-build.stderr"
