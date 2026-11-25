#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/editor-conformance/verify-the-source-built-server-2.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
node tests/editor-conformance/server-artifact.ts verify "${RUNNER_TEMP}/vize-server/vize" "${RUNNER_TEMP}/vize-server/server.json" "$GITHUB_SHA"
chmod +x "${RUNNER_TEMP}/vize-server/vize"
echo "VIZE_SERVER_PATH=${RUNNER_TEMP}/vize-server/vize" >> "$GITHUB_ENV"
