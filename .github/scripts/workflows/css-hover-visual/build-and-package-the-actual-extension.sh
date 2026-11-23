#!/usr/bin/env bash
vp run --workspace-root build:vscode-extension
vp run --workspace-root test:vscode-extension:vsix
