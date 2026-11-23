#!/usr/bin/env bash
cargo clippy --workspace -- -D warnings -D clippy::wildcard_imports && cargo clippy -p vize_maestro --tests -- -D warnings && node tools/commands/ci/github/check-maestro-feature-contract.ts && node tools/commands/ci/check-workspace-lints.ts --check && node tools/commands/ci/check-skeleton-todos.ts --check
