#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/03-source-comparison/scripts/validate-base-metadata.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
cargo metadata --manifest-path base/Cargo.toml --format-version 1 --no-deps > /dev/null
