#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/full-qualification/clippy.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
cargo clippy --workspace -- -D warnings -D clippy::wildcard_imports && cargo clippy -p vize_maestro --tests -- -D warnings && node tools/commands/ci/github/check-maestro-feature-contract.ts && node tools/commands/ci/check-workspace-lints.ts --check && node tools/commands/ci/check-skeleton-todos.ts --check
