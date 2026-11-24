#!/usr/bin/env bash
cargo build --profile ci-opt -p vize --features profiling
vp exec node tests/differential/build-receipt.ts --profile-ci-opt
