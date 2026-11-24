#!/usr/bin/env bash
cargo build --locked --profile ci -p vize
node tests/differential/build-receipt.ts
