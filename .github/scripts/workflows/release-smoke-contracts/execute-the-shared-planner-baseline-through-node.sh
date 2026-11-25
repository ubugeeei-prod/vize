#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/release-smoke-contracts/execute-the-shared-planner-baseline-through-node.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
for phase in cold warm; do
  /usr/bin/time --verbose --output="$RUNNER_TEMP/node-planner-$phase.txt" \
    node --test tests/tooling/legacy-fix-fixture-report.test.ts
done
cat "$RUNNER_TEMP"/node-planner-*.txt
