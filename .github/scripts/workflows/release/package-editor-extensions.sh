#!/usr/bin/env bash
vp run --workspace-root package:vscode-extension
vp run --workspace-root check:zed-extension
vp run --workspace-root package:zed-extension
vp run --workspace-root package:nvim-extension
vp run --workspace-root package:vim-extension
vp run --workspace-root package:helix-extension
vp run --workspace-root package:emacs-extension
