#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/08-vitest/scripts/build-both-source-configuration-adapters.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
(cd npm/cli && vp pack)
(cd npm/builder/vite && vp pack)
