#!/usr/bin/env bash
cargo tree --locked -p vize_test_runner -e features -i vize_canon --prefix none > "$RUNNER_TEMP/canon-portable-features.txt"
if rg --quiet --fixed-strings 'vize_canon feature "native"' "$RUNNER_TEMP/canon-portable-features.txt"; then
  echo 'The portable consumer unexpectedly enables Canon native features.' >&2
  exit 1
fi
cargo test --locked -p vize_test_runner --test portable_canon
