#!/usr/bin/env bash
set -euo pipefail
mkdir -p "$RUNNER_TEMP/cold-paired"
exclusions=(--exclude='*/project' --exclude='*/xdg-cache' --exclude='*/tmp' --exclude='*/binaries' --exclude='*/binary' --exclude='*/target')
if [[ "${PAIRED_ROUTE:-}" == candidate ]]; then
  exclusions+=(--exclude='*/input-blobs' --exclude='*/native-profile-data' --exclude='*/custody-images' --exclude='*/allocation/*/vize')
fi
tar "${exclusions[@]}" -czf "$RUNNER_TEMP/cold-paired-evidence.tgz" -C "$RUNNER_TEMP" cold-paired
