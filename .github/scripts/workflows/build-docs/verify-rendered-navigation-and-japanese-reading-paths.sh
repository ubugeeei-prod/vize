#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/build-docs/verify-rendered-navigation-and-japanese-reading-paths.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
ja_reading_routes=$(node "$VIZE_CI_FILES/verify-rendered-navigation-and-japanese-reading-paths.ts")
node docs/scripts/verify-navigation-render.ts --routes "$ja_reading_routes"
node docs/scripts/verify-ja-editorial-render.ts
