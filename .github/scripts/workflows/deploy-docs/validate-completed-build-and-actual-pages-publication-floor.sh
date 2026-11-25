#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/deploy-docs/validate-completed-build-and-actual-pages-publication-floor.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
# Preserve trusted main validators when the completed source is older.
validator="$RUNNER_TEMP/docs-deployment/compat/github"
mkdir -p "$validator" "$RUNNER_TEMP/docs-deployment/archive"
cp tools/support/compat/github/docs-deployment-*.ts "$validator/"
cp tools/support/archive/tar.ts tools/support/archive/zip.ts "$RUNNER_TEMP/docs-deployment/archive/"
vp node "$validator/docs-deployment-entry.ts" prepare
