#!/usr/bin/env bash
set -euo pipefail
node tools/benchmarks/scripts/canon-slot-constructor-repair.ts "$RUNNER_TEMP/slot-provider" "$RUNNER_TEMP/slot-provider/node_modules/vize/bin/vize" "$RUNNER_TEMP/slot-head" "$RUNNER_TEMP/slot-evidence/native"
