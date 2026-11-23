#!/usr/bin/env bash
cargo build --locked --profile ci -p vize --features legacy
node tests/differential/build-receipt.ts --legacy
