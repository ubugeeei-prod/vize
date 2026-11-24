#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/cold-typechecker-scale/preserve-every-raw-pair-native-observation-control-and-source-receipt.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
mkdir -p "$RUNNER_TEMP/cold-paired"
exclusions=(--exclude='*/project' --exclude='*/xdg-cache' --exclude='*/tmp' --exclude='*/binaries' --exclude='*/binary' --exclude='*/target')
if [[ "${PAIRED_ROUTE:-}" == candidate ]]; then
  exclusions+=(--exclude='*/input-blobs' --exclude='*/native-profile-data' --exclude='*/custody-images' --exclude='*/allocation/*/vize')
fi
tar "${exclusions[@]}" -czf "$RUNNER_TEMP/cold-paired-evidence.tgz" -C "$RUNNER_TEMP" cold-paired
