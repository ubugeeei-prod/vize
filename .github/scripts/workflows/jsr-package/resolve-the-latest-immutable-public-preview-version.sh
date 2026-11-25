#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/jsr-package/resolve-the-latest-immutable-public-preview-version.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
echo "version=$(node tools/support/release/jsr/package.ts latest-public)" >> "$GITHUB_OUTPUT"
