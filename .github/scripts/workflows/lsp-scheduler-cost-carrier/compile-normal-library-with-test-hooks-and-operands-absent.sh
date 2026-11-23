#!/usr/bin/env bash
set -euo pipefail
cargo check --locked --profile ci -p vize_maestro --lib 2>&1 | tee scheduler-evidence/normal-library.log
cargo check --locked --profile ci -p vize_maestro --lib --features micro-profiling 2>&1 | tee scheduler-evidence/normal-library-micro-feature.log
