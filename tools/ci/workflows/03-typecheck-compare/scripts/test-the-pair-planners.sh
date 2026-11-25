#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/03-typecheck-compare/scripts/test-the-pair-planners.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
node --test \
  tests/tooling/typecheck-cold-pair.test.ts \
  tests/tooling/content-mapper-pair.test.ts \
  tests/tooling/content-mapper-replay.test.ts
