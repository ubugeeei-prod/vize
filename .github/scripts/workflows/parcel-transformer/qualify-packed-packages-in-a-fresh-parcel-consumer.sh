#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/parcel-transformer/qualify-packed-packages-in-a-fresh-parcel-consumer.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
test ! -e "$RUNNER_TEMP/parcel-cpu"
mkdir "$RUNNER_TEMP/parcel-cpu"
VIZE_PARCEL_PROFILE_DIR="$RUNNER_TEMP/parcel-cpu" vp run --no-cache --filter './npm/builder/parcel' test:host -- "$RUNNER_TEMP/parcel-proof"
