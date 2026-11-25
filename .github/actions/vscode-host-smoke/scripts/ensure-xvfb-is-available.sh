#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/actions/vscode-host-smoke/scripts/ensure-xvfb-is-available.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
if ! command -v xvfb-run >/dev/null 2>&1; then
  vize-ci-apt-retry sudo apt-get update
  vize-ci-apt-retry sudo apt-get install -y xvfb
fi
