#!/usr/bin/env bash
set -euo pipefail
PARTITION_WEIGHT_BUILD_RECEIPT="$RUNNER_TEMP/partition-weight/binary.json" node tools/benchmarks/scripts/partition-weight-cli.ts \
  "$RUNNER_TEMP/partition-weight-target/ci-opt/vize" "$RUNNER_TEMP/partition-provider" \
  "$RUNNER_TEMP/partition-weight/packets"
