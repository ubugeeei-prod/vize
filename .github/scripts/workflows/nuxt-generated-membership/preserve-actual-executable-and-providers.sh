#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/nuxt-generated-membership/preserve-actual-executable-and-providers.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
binary="${CARGO_TARGET_DIR:-target}/ci/vize"
test -f "$binary"
if [[ -f "$binary" ]]; then
  sha256sum "$binary" > "$RUNNER_TEMP/nuxt-membership-packets/binary-sha256.txt"
  "$binary" --version > "$RUNNER_TEMP/nuxt-membership-packets/binary-version.txt"
fi
node "$VIZE_CI_FILES/preserve-actual-executable-and-providers.cjs" > "$RUNNER_TEMP/nuxt-membership-packets/providers.txt"
