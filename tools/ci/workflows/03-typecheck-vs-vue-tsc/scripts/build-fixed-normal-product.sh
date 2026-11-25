#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/03-typecheck-vs-vue-tsc/scripts/build-fixed-normal-product.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
node tools/benchmarks/scripts/cold-typechecker-scale-build.ts \
  --source-root "$RUNNER_TEMP/comparative/source" \
  --target-dir "$RUNNER_TEMP/comparative/target" \
  --binary "$RUNNER_TEMP/comparative/vize" \
  --receipt "$RUNNER_TEMP/comparative/vize.build.json" \
  --role normal
