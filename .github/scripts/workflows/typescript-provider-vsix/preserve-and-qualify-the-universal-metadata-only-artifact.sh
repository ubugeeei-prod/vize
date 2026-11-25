#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/typescript-provider-vsix/preserve-and-qualify-the-universal-metadata-only-artifact.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
vp run --workspace-root package:vscode-extension
mkdir -p "$VIZE_TYPESCRIPT_PROVIDER_OUTPUT"
cp editors/vscode/dist/vize.vsix "$VIZE_TYPESCRIPT_PROVIDER_OUTPUT/vize.vsix"
