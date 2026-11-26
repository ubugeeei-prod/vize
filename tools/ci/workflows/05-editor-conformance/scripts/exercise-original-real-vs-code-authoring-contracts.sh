#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/05-editor-conformance/scripts/exercise-original-real-vs-code-authoring-contracts.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
mkdir -p "${RUNNER_TEMP}/rich-host-evidence"
cp "${RUNNER_TEMP}/vize-server/vize-producer.txt" "${RUNNER_TEMP}/rich-host-evidence/"
sha256sum "${VIZE_SERVER_PATH}" > "${RUNNER_TEMP}/rich-host-evidence/consumed-server.sha256"
git rev-parse HEAD > "${RUNNER_TEMP}/rich-host-evidence/harness-sha.txt"
xvfb-run -a vp run --workspace-root test:vscode-extension:host-real
