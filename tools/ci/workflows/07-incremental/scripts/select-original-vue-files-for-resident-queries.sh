#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/07-incremental/scripts/select-original-vue-files-for-resident-queries.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
git ls-files -z -- '*.vue' '*.vue.txt' > "$RUNNER_TEMP/resident-block-query-files"
