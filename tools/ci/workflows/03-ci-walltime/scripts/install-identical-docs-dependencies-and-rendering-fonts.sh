#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/03-ci-walltime/scripts/install-identical-docs-dependencies-and-rendering-fonts.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
node tools/support/github/vp-install.ts --filter './docs...' --filter './npm/builder/vite...' --filter './npm/cli...' --filter './npm/native...' --filter './npm/ui...' --filter './npm/compose/core...'
sudo apt-get update
sudo apt-get install -y --no-install-recommends fonts-noto-cjk
