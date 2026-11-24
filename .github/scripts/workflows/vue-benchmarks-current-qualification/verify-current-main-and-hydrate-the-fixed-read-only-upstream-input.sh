#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/vue-benchmarks-current-qualification/verify-current-main-and-hydrate-the-fixed-read-only-upstream-input.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
test "$(git rev-parse HEAD)" = "$EXPECTED_SOURCE_SHA"
test "$(git ls-remote origin refs/heads/main | cut -f1)" = "$EXPECTED_SOURCE_SHA"
test "$(git ls-tree HEAD tests/_fixtures/_git/vue-benchmarks | cut -f1)" = '160000 commit 5489aee433cd1054b9d72973457498544da7c467'
git submodule update --init --depth 1 -- tests/_fixtures/_git/vue-benchmarks
test "$(git -C tests/_fixtures/_git/vue-benchmarks rev-parse HEAD)" = 5489aee433cd1054b9d72973457498544da7c467
test -z "$(git -C tests/_fixtures/_git/vue-benchmarks status --porcelain --untracked-files=all)"
