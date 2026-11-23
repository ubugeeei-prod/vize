#!/usr/bin/env bash
node --test tests/tooling/defaulted-inject-snapshot-transition.test.ts
node tools/support/compat/defaulted-inject/verify.ts
