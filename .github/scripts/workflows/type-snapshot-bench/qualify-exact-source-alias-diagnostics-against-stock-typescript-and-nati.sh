#!/usr/bin/env bash
node head/tools/benchmarks/scripts/canon-alias-native.mjs \
  "$GITHUB_WORKSPACE/base/target/ci-opt/vize" \
  "$GITHUB_WORKSPACE/head/target/ci-opt/vize" \
  "$GITHUB_WORKSPACE/canon-path-alias-before-after.json"
