#!/usr/bin/env bash
set -euo pipefail
mkdir -p tests/node_modules
ln -s "$RUNNER_TEMP/slot-provider-reference/node_modules/typescript" tests/node_modules/typescript
ln -s "$RUNNER_TEMP/slot-provider-reference/node_modules/vue" tests/node_modules/vue
node --test tests/tooling/canon-slot-provider-any.test.ts tests/tooling/canon-registered-slot-provider.test.ts tests/tooling/canon-slot-provider-cli.test.ts
node tools/benchmarks/scripts/canon-slot-provider-any.ts "$RUNNER_TEMP/slot-provider-reference" "$GITHUB_WORKSPACE/target/ci/vize" "$RUNNER_TEMP/slot-provider-any/proof"
node tools/benchmarks/scripts/canon-registered-slot-provider.ts "$RUNNER_TEMP/slot-provider-reference" "$GITHUB_WORKSPACE/target/ci/vize" "$RUNNER_TEMP/slot-provider-any/registered-proof"
node tools/benchmarks/scripts/canon-slot-provider-collisions.ts "$RUNNER_TEMP/slot-provider-reference" "$GITHUB_WORKSPACE/target/ci/vize" "$RUNNER_TEMP/slot-provider-any/collision-proof"
