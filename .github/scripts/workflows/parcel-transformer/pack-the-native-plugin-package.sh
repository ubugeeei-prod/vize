#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/parcel-transformer/pack-the-native-plugin-package.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
node "$RUNNER_TEMP/native-plugin/pack-main.mjs" \
  --dir "$PLUGIN_DIR" \
  --artifacts "$RUNNER_TEMP/artifacts" \
  --out "$RUNNER_TEMP/main"
