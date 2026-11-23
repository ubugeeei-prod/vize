#!/usr/bin/env bash
cargo build --profile ci -p vize && vp node tests/differential/build-receipt.ts
