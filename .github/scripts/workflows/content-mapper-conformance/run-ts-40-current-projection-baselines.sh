#!/usr/bin/env bash
cargo test -p vize --test integration -- davinci_ts40_projection_cli:: --nocapture
cargo test -p vize_maestro --test davinci_ts40_projection -- --nocapture
cargo test -p vize_maestro --features legacy --test davinci_ts40_projection -- --nocapture
node --test tests/tooling/davinci-ts40-projection.test.ts
