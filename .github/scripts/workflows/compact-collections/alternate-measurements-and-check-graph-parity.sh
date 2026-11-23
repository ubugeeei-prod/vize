#!/usr/bin/env bash
mkdir -p measurements
for pass in 1 2 3 4 5; do
  if (( pass % 2 )); then
    head/target/base/release/examples/compact_collections "measurements/base-$pass.json"
    head/target/head/release/examples/compact_collections "measurements/head-$pass.json"
  else
    head/target/head/release/examples/compact_collections "measurements/head-$pass.json"
    head/target/base/release/examples/compact_collections "measurements/base-$pass.json"
  fi
done
node "$VIZE_CI_FILES/alternate-measurements-and-check-graph-parity.ts"
