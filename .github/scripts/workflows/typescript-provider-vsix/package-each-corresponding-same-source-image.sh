#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/typescript-provider-vsix/package-each-corresponding-same-source-image.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
node tools/support/editors/typescript-provider/package.ts x86_64-unknown-linux-gnu .native-artifacts "$VIZE_TYPESCRIPT_PROVIDER_OUTPUT/vize-linux-x64.vsix"
node tools/support/editors/typescript-provider/package.ts x86_64-pc-windows-msvc .native-artifacts "$VIZE_TYPESCRIPT_PROVIDER_OUTPUT/vize-win32-x64.vsix"
