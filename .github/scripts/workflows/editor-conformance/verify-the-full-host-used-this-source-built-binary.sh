#!/usr/bin/env bash
node tests/editor-conformance/server-artifact.ts scorecard "${RUNNER_TEMP}/vize-server/server.json" "${RUNNER_TEMP}/vscode-full-host/host-result.json" "$GITHUB_SHA"
