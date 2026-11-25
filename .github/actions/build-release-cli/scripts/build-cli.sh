#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/actions/build-release-cli/scripts/build-cli.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
if [[ "$RELEASE_TARGET" == *-unknown-linux-gnu ]]; then
  floor_target="$(node tools/support/github/glibc-floor.ts target "$RELEASE_TARGET")"
  cargo zigbuild --locked --release -p vize --target "$floor_target"
  node tools/ci/commands/github/verify-glibc-symbols.ts "target/$RELEASE_TARGET/release/vize"
else
  cargo build --locked --release -p vize --target "$RELEASE_TARGET"
fi
