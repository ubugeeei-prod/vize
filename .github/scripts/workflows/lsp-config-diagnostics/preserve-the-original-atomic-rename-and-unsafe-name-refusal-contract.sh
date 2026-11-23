#!/usr/bin/env bash
set +e
cargo test --locked --profile ci -p vize --features profiling --message-format=json --test integration -- lsp_component_attribute_authority_cli::library_refusal::original_emit_string_keeps_atomic_owned_updates_and_unsafe_name_refusals_without_file_writes --exact --nocapture > target/vize-tests/lsp-config-diagnostics/rename.stdout 2> target/vize-tests/lsp-config-diagnostics/rename.stderr
status=$?
printf '%s\n' "$status" > target/vize-tests/lsp-config-diagnostics/rename.status
node tools/support/compat/github/run-lsp-config-diagnostics.ts --retain-rename-producer
retain_status=$?
cat target/vize-tests/lsp-config-diagnostics/rename.stdout
cat target/vize-tests/lsp-config-diagnostics/rename.stderr >&2
if [ "$status" -ne 0 ]; then
  exit "$status"
fi
exit "$retain_status"
