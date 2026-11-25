#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/benchmark/validate-base-metadata.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
cargo metadata --manifest-path base/Cargo.toml --format-version 1 --no-deps > /dev/null
