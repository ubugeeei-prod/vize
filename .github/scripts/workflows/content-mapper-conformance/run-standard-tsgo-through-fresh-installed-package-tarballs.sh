#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/content-mapper-conformance/run-standard-tsgo-through-fresh-installed-package-tarballs.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
node tools/commands/release/npm/smoke-release-install.ts --prepare-manifests --content-mapper-checks \
  npm/native npm/native/npm/* \
  npm/cli
