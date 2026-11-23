#!/usr/bin/env bash
binary="${CARGO_TARGET_DIR:-target}/ci/vize"
test -f "$binary"
if [[ -f "$binary" ]]; then
  sha256sum "$binary" > "$RUNNER_TEMP/nuxt-membership-packets/binary-sha256.txt"
  "$binary" --version > "$RUNNER_TEMP/nuxt-membership-packets/binary-version.txt"
fi
node "$VIZE_CI_FILES/preserve-actual-executable-and-providers.cjs" > "$RUNNER_TEMP/nuxt-membership-packets/providers.txt"
