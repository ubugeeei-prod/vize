#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/level-instruction-counts/measure-both-parsers-three-times-and-enforce-parity.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
node tools/benchmarks/scripts/instruction-counts.mjs --sfc-parsing --collect --out "${RUNNER_TEMP}/sfc-instructions"
node tools/benchmarks/scripts/sfc-parse-instructions.ts "${RUNNER_TEMP}/sfc-instructions/measurement.json"
