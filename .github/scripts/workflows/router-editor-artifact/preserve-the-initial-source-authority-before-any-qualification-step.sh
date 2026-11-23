#!/usr/bin/env bash
mkdir -p "$RUNNER_TEMP/router-editor-artifact"
git rev-parse HEAD > "$RUNNER_TEMP/router-editor-artifact/source-sha.txt"
git rev-parse 'HEAD^{tree}' > "$RUNNER_TEMP/router-editor-artifact/source-tree.txt"
