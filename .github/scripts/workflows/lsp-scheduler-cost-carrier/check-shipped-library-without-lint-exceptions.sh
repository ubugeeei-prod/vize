#!/usr/bin/env bash
set -euo pipefail
cargo clippy --locked --profile ci -p vize_maestro --lib -- -D warnings 2>&1 | tee scheduler-evidence/normal-clippy.log
