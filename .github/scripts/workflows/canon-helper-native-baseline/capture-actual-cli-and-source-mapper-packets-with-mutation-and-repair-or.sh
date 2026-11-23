#!/usr/bin/env bash
set -euo pipefail
node tools/benchmarks/scripts/canon-helper-baseline.ts "$RUNNER_TEMP/helper-provider" "$RUNNER_TEMP/helper-image/target/ci/vize" "$RUNNER_TEMP/helper-baseline-custody/regeneration.json" "$GITHUB_WORKSPACE/.admission-driver" "$RUNNER_TEMP/helper-baseline" "${VIZE_MATRIX_SFCS}"
