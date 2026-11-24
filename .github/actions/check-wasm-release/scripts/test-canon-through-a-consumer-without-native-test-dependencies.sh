#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/actions/check-wasm-release/scripts/test-canon-through-a-consumer-without-native-test-dependencies.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
cargo tree --locked -p vize_test_runner -e features -i vize_canon --prefix none > "$RUNNER_TEMP/canon-portable-features.txt"
if rg --quiet --fixed-strings 'vize_canon feature "native"' "$RUNNER_TEMP/canon-portable-features.txt"; then
  echo 'The portable consumer unexpectedly enables Canon native features.' >&2
  exit 1
fi
cargo test --locked -p vize_test_runner --test portable_canon
