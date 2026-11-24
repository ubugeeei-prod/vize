#!/usr/bin/env bash
node --test \
  tests/tooling/release/release-smoke-public-native.test.ts \
  tests/tooling/native-build-token.test.ts \
  tests/tooling/native-binding-identity.test.ts
