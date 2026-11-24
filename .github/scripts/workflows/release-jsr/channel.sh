#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/release-jsr/channel.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
node tools/support/release/jsr/package.ts channel >> "$GITHUB_OUTPUT"
