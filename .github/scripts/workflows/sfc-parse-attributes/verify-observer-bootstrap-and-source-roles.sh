#!/usr/bin/env bash
node --test tests/tooling/sfc-parse-attributes-bootstrap.test.ts
node --test tests/tooling/sfc-parse-attributes-reference.test.ts
node tests/native/sfc-parse-attributes-source.mjs
