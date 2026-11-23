#!/usr/bin/env bash
export RUSTFLAGS="${RUSTFLAGS:-} -C force-frame-pointers=yes"
vp run --workspace-root build:native
