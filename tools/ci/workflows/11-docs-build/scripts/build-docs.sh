#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/11-docs-build/scripts/build-docs.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
vp run --no-cache --filter './docs' build
test -s docs-render-evidence/component-previews/browser-evidence.json
test -s docs-render-evidence/component-previews/unavailable-storage-evidence.json
