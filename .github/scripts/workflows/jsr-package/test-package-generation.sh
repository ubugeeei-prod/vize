#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/jsr-package/test-package-generation.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
node --test \
  tests/tooling/jsr-package.test.ts \
  tests/tooling/jsr-public-preview-consumers.test.ts \
  tests/tooling/release-jsr-workflow.test.ts \
  tests/tooling/jsr-fresh-provenance.test.ts
