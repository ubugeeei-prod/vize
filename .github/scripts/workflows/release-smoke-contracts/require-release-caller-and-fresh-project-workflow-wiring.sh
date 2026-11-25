#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/release-smoke-contracts/require-release-caller-and-fresh-project-workflow-wiring.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
node --test \
  tests/tooling/github-workflows-release-build.test.ts \
  tests/tooling/release/release-workflow.test.ts \
  tests/tooling/release/release-smoke-init-fresh.test.ts \
  tests/tooling/release/release-smoke-init-toolchain.test.ts \
  tests/tooling/release/release-smoke-init-workspace.test.ts \
  tests/tooling/release/release-smoke-musl.test.ts
