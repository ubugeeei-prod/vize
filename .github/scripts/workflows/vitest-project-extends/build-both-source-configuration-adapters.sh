#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/vitest-project-extends/build-both-source-configuration-adapters.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
(cd npm/cli && vp pack)
(cd npm/builder/vite && vp pack)
