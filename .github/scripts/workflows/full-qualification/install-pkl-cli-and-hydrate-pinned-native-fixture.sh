#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/full-qualification/install-pkl-cli-and-hydrate-pinned-native-fixture.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
npm install --global @pkl-community/pkl@0.27.2 && git submodule update --init --force --depth 1 -- tests/_fixtures/_git/element-plus
