#!/usr/bin/env bash
node --test \
  tests/tooling/typecheck-cold-pair.test.ts \
  tests/tooling/content-mapper-pair.test.ts \
  tests/tooling/content-mapper-replay.test.ts
