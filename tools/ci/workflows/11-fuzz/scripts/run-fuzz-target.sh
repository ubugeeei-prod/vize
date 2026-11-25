#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/11-fuzz/scripts/run-fuzz-target.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
if [[ "$FUZZ_MODE" == "replay" ]]; then
  # Regression check: replay the seeded and cached corpus once each.
  budget="-runs=0"
else
  budget="-max_total_time=$FUZZ_MAX_TOTAL_TIME"
fi
cargo +nightly fuzz run --fuzz-dir tests/fuzz "$FUZZ_TARGET" -- \
  "$budget" \
  -max_len=65536 \
  -timeout=25 \
  -rss_limit_mb=4096 \
  -malloc_limit_mb=2048
