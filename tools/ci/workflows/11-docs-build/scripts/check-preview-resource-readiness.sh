#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/11-docs-build/scripts/check-preview-resource-readiness.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
vp node docs/scripts/ensure-browser.ts
mkdir -p docs-render-evidence/resource-readiness
node --test --test-reporter=tap tests/tooling/docs-preview-readiness-browser.test.ts 2>&1 | tee docs-render-evidence/resource-readiness/tests.tap
node tools/support/github/docs-preview-readiness.ts \
  docs-render-evidence/resource-readiness/tests.tap \
  docs-render-evidence/resource-readiness
