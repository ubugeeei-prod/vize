#!/usr/bin/env bash
mkdir -p "$RUNNER_TEMP/projection-import-geometry"
git rev-parse HEAD > "$RUNNER_TEMP/projection-import-geometry/source.sha"
args=()
if [[ "$FEATURE" == legacy ]]; then args+=(--features legacy); fi
set -o pipefail
cargo test --locked --profile ci -p vize_maestro "${args[@]}" --test davinci_ts40_projection -- --nocapture 2>&1 | tee "$RUNNER_TEMP/projection-import-geometry/execution.log"
