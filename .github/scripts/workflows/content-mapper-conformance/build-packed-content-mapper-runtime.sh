#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/content-mapper-conformance/build-packed-content-mapper-runtime.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
vp run --filter './npm/native' build:ci
(cd npm/cli && vp pack)
