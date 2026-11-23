#!/usr/bin/env bash
set +e
cargo test --locked --profile ci -p vize --features legacy --message-format=json --test integration -- lsp_completion_sibling_overlay_cli:: --nocapture > target/vize-tests/lsp-config-diagnostics/completion.stdout 2> target/vize-tests/lsp-config-diagnostics/completion.stderr
status=$?
set -e
printf '%s\n' "$status" > target/vize-tests/lsp-config-diagnostics/completion.status
node tools/support/compat/github/run-lsp-config-diagnostics.ts --retain-completion-producer
cat target/vize-tests/lsp-config-diagnostics/completion.stdout
cat target/vize-tests/lsp-config-diagnostics/completion.stderr >&2
exit "$status"
