#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/parcel-transformer/pack-the-native-plugin-target.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
node "$RUNNER_TEMP/native-plugin/pack-artifact.mjs" \
  --dir "$PLUGIN_DIR" \
  --target "$TARGET" \
  --library "$LIBRARY" \
  --out "$RUNNER_TEMP/artifacts"
