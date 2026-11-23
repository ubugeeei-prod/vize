#!/usr/bin/env bash
cargo build --profile ci-opt -p vize
cp target/ci-opt/vize "$RUNNER_TEMP/vize-head"
