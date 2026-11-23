#!/usr/bin/env bash
if [ "$VIZE_WARMUP_ROLE" = baseline ]; then
  node --test --test-concurrency=1 tests/tooling/lsp-native-diagnostic-warmup.test.ts
else
  # Exercise the editor initialization setting itself, without an env override.
  env -u VIZE_LSP_NATIVE_DIAGNOSTIC_TIMEOUT_MS node --test --test-concurrency=1 tests/tooling/lsp-native-diagnostic-warmup.test.ts
fi
