#!/usr/bin/env bash
vp run --workspace-root package:vscode-extension
vp node tests/typescript-plugin/assert-vsix.ts
