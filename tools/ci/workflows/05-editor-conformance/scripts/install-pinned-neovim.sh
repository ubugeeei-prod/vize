#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/05-editor-conformance/scripts/install-pinned-neovim.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
bash .github/actions/vscode-host-smoke/download-artifact.sh \
  "https://github.com/neovim/neovim/releases/download/${NVIM_VERSION}/nvim-linux-x86_64.tar.gz" \
  "${RUNNER_TEMP}/nvim.tar.gz" "${NVIM_SHA256}"
tar -C "${RUNNER_TEMP}" -xzf "${RUNNER_TEMP}/nvim.tar.gz"
echo "VIZE_TEST_NVIM_PATH=${RUNNER_TEMP}/nvim-linux-x86_64/bin/nvim" >> "$GITHUB_ENV"
"${RUNNER_TEMP}/nvim-linux-x86_64/bin/nvim" --version | head -1
