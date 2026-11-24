#!/usr/bin/env bash
node harness/tools/benchmarks/scripts/sfc-parse-replay.mjs \
  "$GITHUB_WORKSPACE/base" "$GITHUB_WORKSPACE/head" \
  "$GITHUB_WORKSPACE/sfc-parse-replay" "${VIZE_INPUTS_REPETITION}"
