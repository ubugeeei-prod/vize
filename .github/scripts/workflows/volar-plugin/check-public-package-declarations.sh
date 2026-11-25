#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/volar-plugin/check-public-package-declarations.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
node "$RUNNER_TEMP/volar-plugin-deps/node_modules/typescript/bin/tsc" \
  --ignoreConfig \
  --strict \
  --noEmit \
  --module NodeNext \
  --moduleResolution NodeNext \
  --target ESNext npm/volar/src/index.d.ts
