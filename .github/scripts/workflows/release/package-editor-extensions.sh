#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/release/package-editor-extensions.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
vp run --workspace-root package:vscode-extension
vp run --workspace-root check:zed-extension
vp run --workspace-root package:zed-extension
vp run --workspace-root package:nvim-extension
vp run --workspace-root package:vim-extension
vp run --workspace-root package:helix-extension
vp run --workspace-root package:emacs-extension
