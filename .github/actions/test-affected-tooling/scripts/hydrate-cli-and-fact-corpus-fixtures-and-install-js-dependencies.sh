#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/actions/test-affected-tooling/scripts/hydrate-cli-and-fact-corpus-fixtures-and-install-js-dependencies.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
git submodule update --init --force --depth 1 -- tests/_fixtures/_git/ant-design-vue tests/_fixtures/_git/create-vue tests/_fixtures/_git/layoutit-grid tests/_fixtures/_git/vue-select tests/_fixtures/_git/vue-router && vp install --frozen-lockfile --prefer-offline
