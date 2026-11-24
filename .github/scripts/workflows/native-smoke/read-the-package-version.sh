#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/native-smoke/read-the-package-version.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
node "$VIZE_CI_FILES/read-the-package-version.cjs" >> "$GITHUB_OUTPUT"
