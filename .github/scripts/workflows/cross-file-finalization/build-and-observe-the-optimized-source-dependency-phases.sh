#!/usr/bin/env bash
cargo build --locked --profile ci-opt -p vize
node tests/tooling/support/cross-file-finalization.ts
