#!/usr/bin/env bash
git submodule update --init --force --depth 1 -- tests/_fixtures/_git/create-vue
test "$(git rev-parse HEAD:tests/_fixtures/_git/create-vue)" = \
  "$(git -C tests/_fixtures/_git/create-vue rev-parse HEAD)"
