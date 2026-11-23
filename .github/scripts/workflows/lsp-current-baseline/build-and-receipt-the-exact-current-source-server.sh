#!/usr/bin/env bash
cargo build --profile ci -p vize
node tests/differential/build-receipt.ts
