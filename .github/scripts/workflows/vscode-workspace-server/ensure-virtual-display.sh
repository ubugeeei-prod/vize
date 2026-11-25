#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/vscode-workspace-server/ensure-virtual-display.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
if ! command -v xvfb-run >/dev/null 2>&1; then
  sudo apt-get update
  sudo apt-get install -y xvfb
fi
