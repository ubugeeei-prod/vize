#!/usr/bin/env bash
if [[ "$RELEASE_TARGET" == *-musl ]]; then
  node tools/commands/ci/github/verify-musl-cli-binary.ts "$RELEASE_TARGET"
fi
node tools/support/github/create-cli-archive.ts "$RELEASE_TARGET" "$RELEASE_ARCHIVE" vize
