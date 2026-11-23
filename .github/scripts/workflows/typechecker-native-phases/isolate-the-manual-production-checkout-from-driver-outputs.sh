#!/usr/bin/env bash
set -euo pipefail
[[ ! -e "$RUNNER_TEMP/native-phase-source" ]]
mv "$GITHUB_WORKSPACE/.native-phase-source" "$RUNNER_TEMP/native-phase-source"
echo "NATIVE_PHASE_SOURCE_ROOT=$RUNNER_TEMP/native-phase-source" >> "$GITHUB_ENV"
