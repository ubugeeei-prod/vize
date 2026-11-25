#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/actions/test-native-canon/scripts/install-locked-vue-dependency-and-pinned-syntax-corpus.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
node tools/support/github/vp-install.ts --filter './npm/builder/vite...' --filter './npm/cli...'
git submodule update --init --depth 1 -- tests/_fixtures/_git/element-plus
