#!/usr/bin/env bash
if [[ "$RELEASE_TARGET" == *-unknown-linux-gnu ]]; then
  floor_target="$(node tools/support/github/glibc-floor.ts target "$RELEASE_TARGET")"
  cargo zigbuild --locked --release -p vize --target "$floor_target"
  node tools/ci/commands/github/verify-glibc-symbols.ts "target/$RELEASE_TARGET/release/vize"
else
  cargo build --locked --release -p vize --target "$RELEASE_TARGET"
fi
