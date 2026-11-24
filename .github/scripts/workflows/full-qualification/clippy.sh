#!/usr/bin/env bash
cargo clippy --workspace -- -D warnings -D clippy::wildcard_imports && cargo clippy -p vize_maestro --tests -- -D warnings && node tools/ci/commands/github/check-maestro-feature-contract.ts && node tools/ci/commands/check-workspace-lints.ts --check && node tools/ci/commands/check-skeleton-todos.ts --check
