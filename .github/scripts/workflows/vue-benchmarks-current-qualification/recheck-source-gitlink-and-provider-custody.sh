#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/vue-benchmarks-current-qualification/recheck-source-gitlink-and-provider-custody.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
test "$(git rev-parse HEAD)" = "$EXPECTED_SOURCE_SHA"
test -z "$(git status --porcelain --untracked-files=no)"
test -z "$(git -C tests/_fixtures/_git/vue-benchmarks status --porcelain --untracked-files=all)"
cmp tools/benchmarks/fixtures/vue-benchmarks-current-typecheck-provider/package-lock.json "$RUNNER_TEMP/vue-benchmarks-current-provider/package-lock.json"
