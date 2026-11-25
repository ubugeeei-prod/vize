#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/tests/tooling/programs/workflow/collect-js-test-cases.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
export PATH="$1:$PATH"
node plain.mjs && node --test a.test.ts
status=$?
vp test run src/one.test.ts
tsx --test b.test.ts
exit "$status"
