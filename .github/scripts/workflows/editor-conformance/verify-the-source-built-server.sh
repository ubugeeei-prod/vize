#!/usr/bin/env bash
set -euo pipefail
(cd "${RUNNER_TEMP}/vize-server" && sha256sum --check vize.sha256)
grep -Fx "producer=${GITHUB_SHA}" "${RUNNER_TEMP}/vize-server/vize-producer.txt"
node tests/editor-conformance/server-artifact.ts verify "${RUNNER_TEMP}/vize-server/vize" "${RUNNER_TEMP}/vize-server/server.json" "$GITHUB_SHA"
chmod +x "${RUNNER_TEMP}/vize-server/vize"
echo "VIZE_SERVER_PATH=${RUNNER_TEMP}/vize-server/vize" >> "$GITHUB_ENV"
