#!/usr/bin/env bash
set -euo pipefail
test -f "$PREBUILT_VSIX"
mkdir -p editors/vscode/dist
install -m 0644 "$PREBUILT_VSIX" editors/vscode/dist/vize.vsix
cmp "$PREBUILT_VSIX" editors/vscode/dist/vize.vsix
