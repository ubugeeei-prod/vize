#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/actions/build-release-cli/scripts/create-archive-unix.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
if [[ "$RELEASE_TARGET" == *-musl ]]; then
  node tools/commands/ci/github/verify-musl-cli-binary.ts "$RELEASE_TARGET"
fi
node tools/support/github/create-cli-archive.ts "$RELEASE_TARGET" "$RELEASE_ARCHIVE" vize
