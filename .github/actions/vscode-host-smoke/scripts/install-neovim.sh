#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/actions/vscode-host-smoke/scripts/install-neovim.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
bash .github/actions/vscode-host-smoke/download-artifact.sh \
  "https://github.com/neovim/neovim/releases/download/${NVIM_VERSION}/nvim-linux-x86_64.tar.gz" \
  "${RUNNER_TEMP}/nvim.tar.gz" "${NVIM_SHA256}"
sudo tar -C /opt -xzf "${RUNNER_TEMP}/nvim.tar.gz"
sudo ln -sf /opt/nvim-linux-x86_64/bin/nvim /usr/local/bin/nvim
nvim --version | head -1
