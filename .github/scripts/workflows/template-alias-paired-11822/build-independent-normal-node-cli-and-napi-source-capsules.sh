#!/usr/bin/env bash
set -euo pipefail
node tools/benchmarks/scripts/template-alias-paired-11822-cpu.ts "$RUNNER_TEMP/mapper-public-stages/cpu-preflight.json"
node tools/benchmarks/scripts/template-alias-paired-11822-source.ts --source "$GITHUB_WORKSPACE/base" --sha "$BASE_SHA" > "$RUNNER_TEMP/mapper-public-stages/base.log" 2>&1
node tools/benchmarks/scripts/template-alias-paired-11822-source.ts --source "$GITHUB_WORKSPACE/candidate" --sha "$HEAD_SHA" > "$RUNNER_TEMP/mapper-public-stages/candidate.log" 2>&1
