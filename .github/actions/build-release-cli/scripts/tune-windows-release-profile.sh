#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/actions/build-release-cli/scripts/tune-windows-release-profile.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
echo "CARGO_PROFILE_RELEASE_LTO=thin" >> "$GITHUB_ENV"
echo "CARGO_PROFILE_RELEASE_CODEGEN_UNITS=16" >> "$GITHUB_ENV"
