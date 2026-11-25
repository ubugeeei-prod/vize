#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/actions/check-vue-parity/scripts/check-glyph-formatter-corpus-properties.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
node --test --test-concurrency=1 \
  tests/tooling/sfc-baseline-routes.test.ts \
  tests/tooling/sfc-baselines.test.ts \
  tests/tooling/vue2-render-signature.test.ts \
  tests/tooling/glyph-sfc-evidence.test.ts \
  tests/tooling/sfc-equivalence.test.ts \
  tests/tooling/glyph-corpus-idempotence.test.ts \
  tests/tooling/glyph-corpus-parse-preservation.test.ts \
  tests/tooling/glyph-pug-oracle.test.ts \
  tests/tooling/glyph-corpus-lint-agreement.test.ts
