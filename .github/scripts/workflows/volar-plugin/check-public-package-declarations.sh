#!/usr/bin/env bash
node "$RUNNER_TEMP/volar-plugin-deps/node_modules/typescript/bin/tsc" \
  --ignoreConfig \
  --strict \
  --noEmit \
  --module NodeNext \
  --moduleResolution NodeNext \
  --target ESNext npm/volar/src/index.d.ts
