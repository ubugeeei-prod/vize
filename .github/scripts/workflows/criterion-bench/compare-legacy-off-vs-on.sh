#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/criterion-bench/compare-legacy-off-vs-on.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
node tools/benchmarks/scripts/dialect-guard.ts \
  --input tools/benchmarks/scripts/__in__ \
  --off-bin target/off/ci-opt/vize \
  --on-bin target/on/ci-opt/vize \
  --out-dir "$GITHUB_WORKSPACE/dialect-guard-out" \
  --threshold "$DIALECT_GUARD_THRESHOLD_PERCENT"
