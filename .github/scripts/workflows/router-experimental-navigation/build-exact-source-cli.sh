#!/usr/bin/env bash
cargo build --profile ci -p vize
vp node tests/differential/build-receipt.ts
cp target/ci/vize.differential-build.json "$RUNNER_TEMP/router-navigation/source-build.json"
sha256sum target/ci/vize > "$RUNNER_TEMP/router-navigation/source-cli.sha256"
