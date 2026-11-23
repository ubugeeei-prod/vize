#!/usr/bin/env bash
cargo build --profile ci -p vize --features legacy
node tests/differential/build-receipt.ts --legacy
