#!/usr/bin/env bash
set -euo pipefail
mkdir "$RUNNER_TEMP/cache-source-checkpoint" "$RUNNER_TEMP/cache-source-build-proof"
gh api repos/ubugeeei-prod/vize/actions/artifacts/12765860439/zip > "$RUNNER_TEMP/cache-source-checkpoint.zip"
gh api repos/ubugeeei-prod/vize/actions/artifacts/12764104945/zip > "$RUNNER_TEMP/cache-source-build-proof.zip"
unzip -q "$RUNNER_TEMP/cache-source-checkpoint.zip" -d "$RUNNER_TEMP/cache-source-checkpoint"
unzip -q "$RUNNER_TEMP/cache-source-build-proof.zip" -d "$RUNNER_TEMP/cache-source-build-proof"
tar -xzf "$RUNNER_TEMP/cache-source-checkpoint/cache-source-image.tgz" -C "$RUNNER_TEMP/cache-source-checkpoint"
tar -xzf "$RUNNER_TEMP/cache-source-build-proof/cache-metadata.tgz" -C "$RUNNER_TEMP/cache-source-build-proof"
node tools/benchmarks/scripts/typechecker-cache-reuse.ts --image "$RUNNER_TEMP/cache-source-checkpoint/target/ci/vize" --build "$RUNNER_TEMP/cache-source-build-proof/cache-evidence/source-build.json" --out "$RUNNER_TEMP/cache-evidence/source-reuse"
