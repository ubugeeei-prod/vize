#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/parcel-transformer/check-the-native-transformer-crate.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
cargo test -p vize_parcel_transformer
cargo clippy -p vize_parcel_transformer --all-targets -- -D warnings
