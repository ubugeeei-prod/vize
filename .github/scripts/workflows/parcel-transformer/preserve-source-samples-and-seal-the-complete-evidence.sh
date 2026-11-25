#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/parcel-transformer/preserve-source-samples-and-seal-the-complete-evidence.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
if test -d "$RUNNER_TEMP/parcel-cpu"; then
  node tools/support/compat/parcel/profile.mjs "$RUNNER_TEMP/parcel-proof" "$RUNNER_TEMP/parcel-cpu"
fi
