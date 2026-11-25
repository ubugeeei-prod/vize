#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/typecheck-cold-pair/test-the-pair-parsers-and-the-byte-probe.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
node --test \
  tests/tooling/typecheck-corpus-pair.test.ts \
  tests/tooling/typecheck-corpus-pair-corpora.test.ts \
  tests/tooling/typecheck-corpus-pair-cold.test.ts \
  tests/tooling/typecheck-corpus-pair-floors.test.ts \
  tests/tooling/typecheck-owned-pair.test.ts \
  tests/tooling/typecheck-corpus-mutations.test.ts
