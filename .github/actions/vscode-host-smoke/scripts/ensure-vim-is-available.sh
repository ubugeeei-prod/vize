#!/usr/bin/env bash
set -euo pipefail
if ! command -v vim || ! vim --version | grep -F '+timers' >/dev/null; then
  vize-ci-apt-retry sudo apt-get update
  vize-ci-apt-retry sudo apt-get install -y vim-nox
fi
vim --version | sed -n '1,2p'
vim --version | grep -F '+timers' >/dev/null
