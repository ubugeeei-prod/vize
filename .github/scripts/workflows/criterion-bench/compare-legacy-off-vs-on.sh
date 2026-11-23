#!/usr/bin/env bash
node tools/benchmarks/scripts/dialect-guard.ts \
  --input tools/benchmarks/scripts/__in__ \
  --off-bin target/off/ci-opt/vize \
  --on-bin target/on/ci-opt/vize \
  --out-dir "$GITHUB_WORKSPACE/dialect-guard-out" \
  --threshold "$DIALECT_GUARD_THRESHOLD_PERCENT"
