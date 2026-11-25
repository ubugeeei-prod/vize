#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/03-vscode-startup/scripts/validate-inputs.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
[[ "$BASE" =~ ^[0-9a-f]{40}$ ]] && [[ "$HEAD" =~ ^[0-9a-f]{40}$ ]]
[[ "$SERVER_VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]
