#!/usr/bin/env bash
export PATH="$1:$PATH"
node plain.mjs && node --test a.test.ts
status=$?
vp test run src/one.test.ts
tsx --test b.test.ts
exit "$status"
