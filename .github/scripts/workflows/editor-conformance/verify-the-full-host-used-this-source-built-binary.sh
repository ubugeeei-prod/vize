#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/editor-conformance/verify-the-full-host-used-this-source-built-binary.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
node tests/editor-conformance/server-artifact.ts scorecard "${RUNNER_TEMP}/vize-server/server.json" "${RUNNER_TEMP}/vscode-full-host/host-result.json" "$GITHUB_SHA"
