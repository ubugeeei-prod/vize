#!/usr/bin/env bash
node --test \
  tests/tooling/typecheck-corpus-pair.test.ts \
  tests/tooling/typecheck-corpus-pair-corpora.test.ts \
  tests/tooling/typecheck-corpus-pair-cold.test.ts \
  tests/tooling/typecheck-corpus-pair-floors.test.ts \
  tests/tooling/typecheck-owned-pair.test.ts \
  tests/tooling/typecheck-corpus-mutations.test.ts
