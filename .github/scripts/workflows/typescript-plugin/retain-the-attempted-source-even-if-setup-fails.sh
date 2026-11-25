#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/typescript-plugin/retain-the-attempted-source-even-if-setup-fails.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
mkdir -p "$RUNNER_TEMP/typescript-plugin"
git rev-parse HEAD > "$RUNNER_TEMP/typescript-plugin/attempted-source.txt"
