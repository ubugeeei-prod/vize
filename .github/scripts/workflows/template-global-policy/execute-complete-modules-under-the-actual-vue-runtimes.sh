#!/usr/bin/env bash
unset NAPI_RS_NATIVE_LIBRARY_PATH
node --test tests/tooling/native-template-global-policy.test.ts
