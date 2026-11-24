#!/usr/bin/env bash
set -euo pipefail
node tools/support/github/vp-install.ts --filter './npm/builder/vite...' --filter './npm/cli...'
git submodule update --init --depth 1 -- tests/_fixtures/_git/element-plus
