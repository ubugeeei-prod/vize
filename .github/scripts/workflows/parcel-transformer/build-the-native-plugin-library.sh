#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/parcel-transformer/build-the-native-plugin-library.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail

# Setting RUSTFLAGS at all shadows build.rustflags from .cargo/config.toml,
# so only export it when a target actually needs extra flags.
if [ -n "$(printf '%s' "$EXTRA_RUSTFLAGS" | tr -d '[:space:]')" ]; then
  export RUSTFLAGS="$EXTRA_RUSTFLAGS"
  echo "RUSTFLAGS=$RUSTFLAGS"
fi

# The JSON message stream says exactly which file cargo produced;
# diagnostics still go to stderr and stay readable.
if [ "$BUILDER" = 'zig' ]; then
  cargo zigbuild --release --target "$ZIG_TARGET" --message-format json-render-diagnostics $CARGO_ARGS > "$RUNNER_TEMP/cargo-messages.json"
else
  cargo build --release --target "$TARGET" --message-format json-render-diagnostics $CARGO_ARGS > "$RUNNER_TEMP/cargo-messages.json"
fi

LIBRARY="$(node "$RUNNER_TEMP/native-plugin/cargo-library.mjs" --dir . --messages "$RUNNER_TEMP/cargo-messages.json" --ext "$EXT")"
echo "LIBRARY=$LIBRARY" >> "$GITHUB_ENV"
