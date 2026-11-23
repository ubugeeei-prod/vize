#!/usr/bin/env bash
[[ "$QUALIFIER_SHA" =~ ^[0-9a-f]{40}$ ]]
test "$(git rev-parse HEAD)" = "$QUALIFIER_SHA"
mkdir -p target/native-syntax-completion
git rev-parse HEAD > target/native-syntax-completion/source.sha
git ls-files -s > target/native-syntax-completion/whole-source-index.txt
