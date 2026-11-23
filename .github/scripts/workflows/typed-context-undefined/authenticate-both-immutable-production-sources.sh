#!/usr/bin/env bash
set -euo pipefail
[[ "$SOURCE_SHA" =~ ^[0-9a-f]{40}$ ]]
test "$(git rev-parse HEAD)" = "$SOURCE_SHA"
test "$(git -C source rev-parse HEAD)" = "$SOURCE_SHA"
test "$(git -C before rev-parse HEAD)" = "$BEFORE_SHA"
directory="$RUNNER_TEMP/typed-context-undefined"
mkdir -p "$directory"
git -C source rev-parse HEAD > "$directory/source.sha"
git -C before rev-parse HEAD > "$directory/before.sha"
git -C source ls-files -s > "$directory/head-source-index.txt"
git -C before ls-files -s > "$directory/before-source-index.txt"
cp -a source/tests/_fixtures/differential/typechecker/template-context-undefined-10667/public public
