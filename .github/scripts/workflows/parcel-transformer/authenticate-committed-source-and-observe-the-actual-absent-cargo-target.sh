#!/usr/bin/env bash
test "$(git rev-parse HEAD)" = "$SOURCE_SHA"
test -z "$(git status --porcelain --untracked-files=normal)"
node tools/support/compat/parcel/native-target.mjs before "$GITHUB_WORKSPACE"
node npm/native/scripts/build-ci.mjs --request-ci-capture
