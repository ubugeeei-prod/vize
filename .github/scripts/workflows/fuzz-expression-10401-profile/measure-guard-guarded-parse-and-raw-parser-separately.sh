#!/usr/bin/env bash
set -euo pipefail
binary="$CARGO_TARGET_DIR/ci-opt/examples/fuzz_expression_10401_profile"
sha256sum "$binary" > evidence/optimized-binary.sha256
"$binary" inspect "$EXPRESSION_INPUT" > evidence/input-inspection.json
"$binary" controls "$EXPRESSION_INPUT" > evidence/controls.json
for phase in guard guarded raw; do
  repetitions=1
  if [[ "$phase" == guard ]]; then repetitions=1000; fi
  timeout 90 "$binary" "$phase" "$EXPRESSION_INPUT" "$repetitions" > "evidence/$phase.json" 2> "evidence/$phase-metadata.json"
done
