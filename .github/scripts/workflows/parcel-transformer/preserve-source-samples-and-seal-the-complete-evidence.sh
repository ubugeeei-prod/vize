#!/usr/bin/env bash
if test -d "$RUNNER_TEMP/parcel-cpu"; then
  node tools/support/compat/parcel/profile.mjs "$RUNNER_TEMP/parcel-proof" "$RUNNER_TEMP/parcel-cpu"
fi
