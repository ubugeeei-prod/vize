#!/usr/bin/env bash
node --test \
  tests/tooling/jsr-package.test.ts \
  tests/tooling/jsr-public-preview-consumers.test.ts \
  tests/tooling/release-jsr-workflow.test.ts \
  tests/tooling/jsr-fresh-provenance.test.ts
