#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/actions/vscode-host-smoke/scripts/restore-and-verify-the-supplied-release-vsix.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
test -f "$PREBUILT_VSIX"
mkdir -p editors/vscode/dist
install -m 0644 "$PREBUILT_VSIX" editors/vscode/dist/vize.vsix
cmp "$PREBUILT_VSIX" editors/vscode/dist/vize.vsix
