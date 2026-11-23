#!/usr/bin/env bash
set -euo pipefail
mkdir -p node_modules
ln -s "$RUNNER_TEMP/provider/node_modules/typescript" node_modules/typescript
node --test tests/tooling/cold-typechecker-scale-native-forward.test.ts tests/tooling/cold-typechecker-scale-paired.test.ts tests/tooling/cold-typechecker-scale-strict-pair.test.ts tests/tooling/cold-typechecker-scale-read-buffer.test.ts tests/tooling/cold-typechecker-scale-candidate-{law,source,inputs,native,archive,allocation,observations,originals,flow,plan}.test.ts tests/tooling/cold-typechecker-scale-{native-command,cpu-report,memory-proto,memory-report}.test.ts
