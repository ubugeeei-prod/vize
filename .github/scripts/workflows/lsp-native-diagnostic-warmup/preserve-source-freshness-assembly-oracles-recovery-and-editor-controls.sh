#!/usr/bin/env bash
cargo test --locked --profile ci -p vize_maestro --lib server::initial_diagnostics::
cargo test --locked --profile ci -p vize_maestro --lib ide::diagnostics::native::timeout::
cargo test --locked --profile ci -p vize_maestro --lib server::diagnostic_publishing::
cargo test --locked --profile ci -p vize_maestro --lib ide::diagnostics::assembly_parity_tests::
cargo test --locked --profile ci -p vize_maestro --lib server::state::config_tests::
cargo clippy --locked --profile ci -p vize_maestro --all-targets -- -D warnings
node --test --test-concurrency=1 tests/tooling/lsp-initial-diagnostics-version.test.ts tests/tooling/lsp-responsive-lint-diagnostics.test.ts tests/tooling/vscode-extension-core-behavior.test.ts tests/tooling/vscode-extension-manifest.test.ts tests/tooling/lsp-startup-local-interactive.test.ts
