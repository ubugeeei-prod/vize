#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/typechecker-native-phases/isolate-the-manual-production-checkout-from-driver-outputs.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
[[ ! -e "$RUNNER_TEMP/native-phase-source" ]]
mv "$GITHUB_WORKSPACE/.native-phase-source" "$RUNNER_TEMP/native-phase-source"
echo "NATIVE_PHASE_SOURCE_ROOT=$RUNNER_TEMP/native-phase-source" >> "$GITHUB_ENV"
