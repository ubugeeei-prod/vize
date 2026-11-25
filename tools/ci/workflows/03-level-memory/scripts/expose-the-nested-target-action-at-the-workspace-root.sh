#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/03-level-memory/scripts/expose-the-nested-target-action-at-the-workspace-root.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
mkdir -p .github/actions && cp -R head/.github/actions/{reuse-rust-target,setup-rust-sticky-cache} .github/actions/
