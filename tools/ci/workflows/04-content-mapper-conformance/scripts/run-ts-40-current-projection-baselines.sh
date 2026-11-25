#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/04-content-mapper-conformance/scripts/run-ts-40-current-projection-baselines.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
cargo test -p vize --test integration -- davinci_ts40_projection_cli:: --nocapture
cargo test -p vize_maestro --test davinci_ts40_projection -- --nocapture
cargo test -p vize_maestro --features legacy --test davinci_ts40_projection -- --nocapture
node --test tests/tooling/davinci-ts40-projection.test.ts
