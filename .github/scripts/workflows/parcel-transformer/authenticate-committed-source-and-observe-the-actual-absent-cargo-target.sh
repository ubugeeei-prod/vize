#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/parcel-transformer/authenticate-committed-source-and-observe-the-actual-absent-cargo-target.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
test "$(git rev-parse HEAD)" = "$SOURCE_SHA"
test -z "$(git status --porcelain --untracked-files=normal)"
node tools/support/compat/parcel/native-target.mjs before "$GITHUB_WORKSPACE"
node npm/native/scripts/build-ci.mjs --request-ci-capture
