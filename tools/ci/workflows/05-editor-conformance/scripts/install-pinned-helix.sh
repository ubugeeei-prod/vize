#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/05-editor-conformance/scripts/install-pinned-helix.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
archive="${RUNNER_TEMP}/helix-${HELIX_VERSION}-x86_64-linux.tar.xz"
bash .github/actions/vscode-host-smoke/download-artifact.sh \
  "https://github.com/helix-editor/helix/releases/download/${HELIX_VERSION}/helix-${HELIX_VERSION}-x86_64-linux.tar.xz" \
  "${archive}" "${HELIX_SHA256}"
mkdir -p "${RUNNER_TEMP}/helix"
tar -xJf "${archive}" -C "${RUNNER_TEMP}/helix" --strip-components=1
{
  echo "VIZE_TEST_HELIX_PATH=${RUNNER_TEMP}/helix/hx"
  echo "HELIX_RUNTIME=${RUNNER_TEMP}/helix/runtime"
} >> "$GITHUB_ENV"
"${RUNNER_TEMP}/helix/hx" --version
