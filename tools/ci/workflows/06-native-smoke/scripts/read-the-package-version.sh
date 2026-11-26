#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/06-native-smoke/scripts/read-the-package-version.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
node "$VIZE_CI_FILES/read-the-package-version.cjs" >> "$GITHUB_OUTPUT"
