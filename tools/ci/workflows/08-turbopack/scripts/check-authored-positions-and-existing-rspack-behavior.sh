#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/08-turbopack/scripts/check-authored-positions-and-existing-rspack-behavior.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
mkdir -p "$VIZE_TURBOPACK_ARTIFACTS"
node --test tests/tooling/turbopack-workflow.test.ts npm/builder/turbopack/test/consumer-packaging.test.mjs
node --test npm/builder/turbopack/src/*.test.ts npm/builder/shared/*.test.ts npm/builder/rspack/src/shared/source-map.test.ts npm/builder/rspack/src/shared/output.test.ts npm/builder/rspack/src/shared/hotReload.test.ts | tee "$VIZE_TURBOPACK_ARTIFACTS/source-unit.tap"
