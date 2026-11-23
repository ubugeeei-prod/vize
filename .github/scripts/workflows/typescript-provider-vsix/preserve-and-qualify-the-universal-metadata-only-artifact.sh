#!/usr/bin/env bash
vp run --workspace-root package:vscode-extension
mkdir -p "$VIZE_TYPESCRIPT_PROVIDER_OUTPUT"
cp editors/vscode/dist/vize.vsix "$VIZE_TYPESCRIPT_PROVIDER_OUTPUT/vize.vsix"
