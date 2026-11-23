#!/usr/bin/env bash
node --test tests/tooling/jsx-child-style-successors.test.ts tests/tooling/jsx-vdom-child-style-successors.test.ts tests/tooling/sfc-style-raw-module-successors.test.ts
node "$VIZE_CI_FILES/compare-complete-packets-and-explicit-authored-module-successors.ts"
