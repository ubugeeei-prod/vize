#!/usr/bin/env bash
mkdir -p "$RUNNER_TEMP/jsx-backend-origins"
test "$(git rev-parse HEAD)" = "$QUALIFICATION_HEAD_SHA"
git diff --exit-code
cargo test --locked --profile ci -p vize_atelier_jsx --test backend_module_origins -- --nocapture
