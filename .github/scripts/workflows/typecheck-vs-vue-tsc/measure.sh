#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/typecheck-vs-vue-tsc/measure.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
node tools/benchmarks/scripts/typecheck-vs-vue-tsc-frozen.ts --source-root "$RUNNER_TEMP/comparative/source" --binary "$RUNNER_TEMP/comparative/vize" --provider "$RUNNER_TEMP/comparative/provider" --out "$RUNNER_TEMP/comparative/observations"
cat "$RUNNER_TEMP/comparative/observations/summary.md" >> "$GITHUB_STEP_SUMMARY"
