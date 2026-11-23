#!/usr/bin/env bash
test "$(git rev-parse HEAD)" = "$QUALIFIER_SHA"
if [[ "${GITHUB_EVENT_NAME}" == workflow_dispatch ]]; then
  test "$RUN_HEAD_SHA" = "$QUALIFIER_SHA"
fi
mkdir -p target/vuefes-ssr-style-dependencies
git show HEAD:npm/native/index.d.ts > target/vuefes-ssr-style-dependencies/original-index.d.ts
node npm/native/scripts/build-ci.mjs --request-ci-capture
