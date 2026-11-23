#!/usr/bin/env bash
node --test head/tools/benchmarks/scripts/patina-plain-type-packets.test.mjs
node head/tools/benchmarks/scripts/patina-plain-type-packets.mjs \
  "$GITHUB_WORKSPACE/base" "$GITHUB_WORKSPACE/head" \
  "$BENCHMARK_BASE_SHA" "$BENCHMARK_HEAD_SHA" \
  "$GITHUB_WORKSPACE/patina-plain-type-packets"
