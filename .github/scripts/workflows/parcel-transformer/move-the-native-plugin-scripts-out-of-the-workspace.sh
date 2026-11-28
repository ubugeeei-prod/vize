#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/parcel-transformer/move-the-native-plugin-scripts-out-of-the-workspace.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
mkdir -p "$RUNNER_TEMP/native-plugin"
mv .parcel-native-plugin-scripts/.github/scripts/native-plugin/* "$RUNNER_TEMP/native-plugin/"
rm -rf .parcel-native-plugin-scripts
