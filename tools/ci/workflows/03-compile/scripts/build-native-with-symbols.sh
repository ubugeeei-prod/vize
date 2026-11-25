#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/03-compile/scripts/build-native-with-symbols.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
export RUSTFLAGS="${RUSTFLAGS:-} -C force-frame-pointers=yes"
vp run --workspace-root build:native
