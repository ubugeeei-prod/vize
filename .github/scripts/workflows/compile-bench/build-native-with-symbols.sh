#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/compile-bench/build-native-with-symbols.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
export RUSTFLAGS="${RUSTFLAGS:-} -C force-frame-pointers=yes"
vp run --workspace-root build:native
