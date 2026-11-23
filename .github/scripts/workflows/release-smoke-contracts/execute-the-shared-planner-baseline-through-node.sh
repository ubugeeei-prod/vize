#!/usr/bin/env bash
for phase in cold warm; do
  /usr/bin/time --verbose --output="$RUNNER_TEMP/node-planner-$phase.txt" \
    node --test tests/tooling/legacy-fix-fixture-report.test.ts
done
cat "$RUNNER_TEMP"/node-planner-*.txt
