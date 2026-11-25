#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/05-editor-conformance/scripts/hydrate-pinned-create-vue-fixture-for-the-original-real-host-suite.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
git submodule update --init --force --depth 1 -- tests/_fixtures/_git/create-vue
test "$(git rev-parse HEAD:tests/_fixtures/_git/create-vue)" = \
  "$(git -C tests/_fixtures/_git/create-vue rev-parse HEAD)"
