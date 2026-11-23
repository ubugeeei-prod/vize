#!/usr/bin/env bash
set -euo pipefail
mkdir -p "$RUNNER_TEMP/workspace-identity"
tar --exclude=cargo-target --exclude=base-cargo-target --exclude=base-source --exclude=runtime -czf "$RUNNER_TEMP/workspace-identity.tgz" -C "$RUNNER_TEMP" workspace-identity
