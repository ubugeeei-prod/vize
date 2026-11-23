#!/usr/bin/env bash
set -euo pipefail
mkdir -p "${RUNNER_TEMP}/zed-extension-scratch"
wasi_archive="${RUNNER_TEMP}/wasi-sdk-25.0-x86_64-linux.tar.gz"
bash "${GITHUB_WORKSPACE}/.github/actions/vscode-host-smoke/download-artifact.sh" \
  "https://github.com/WebAssembly/wasi-sdk/releases/download/wasi-sdk-25/wasi-sdk-25.0-x86_64-linux.tar.gz" \
  "$wasi_archive" \
  "52640dde13599bf127a95499e61d6d640256119456d1af8897ab6725bcf3d89c" \
  "https://api.github.com/repos/WebAssembly/wasi-sdk/releases/assets/212790091"
tar -xzf "$wasi_archive" -C "${RUNNER_TEMP}/zed-extension-scratch"
mv "${RUNNER_TEMP}/zed-extension-scratch/wasi-sdk-25.0-x86_64-linux" \
  "${RUNNER_TEMP}/zed-extension-scratch/wasi-sdk"
test -x "${RUNNER_TEMP}/zed-extension-scratch/wasi-sdk/bin/clang"
mkdir -p "${RUNNER_TEMP}/zed-extension-output"
"${RUNNER_TEMP}/zed-extension" \
  --source-dir . \
  --scratch-dir "${RUNNER_TEMP}/zed-extension-scratch" \
  --output-dir "${RUNNER_TEMP}/zed-extension-output"
mkdir -p "${RUNNER_TEMP}/zed-extension-installed"
tar -xzf "${RUNNER_TEMP}/zed-extension-output/archive.tar.gz" \
  -C "${RUNNER_TEMP}/zed-extension-installed"
