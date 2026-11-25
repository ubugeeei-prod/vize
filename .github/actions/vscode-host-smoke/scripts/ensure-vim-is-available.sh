#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/actions/vscode-host-smoke/scripts/ensure-vim-is-available.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
if ! command -v vim || ! vim --version | grep -F '+timers' >/dev/null; then
  vize-ci-apt-retry sudo apt-get update
  vize-ci-apt-retry sudo apt-get install -y vim-nox
fi
vim --version | sed -n -f "$GITHUB_ACTION_PATH/scripts/vim-version-header.sed"
vim --version | grep -F '+timers' >/dev/null
