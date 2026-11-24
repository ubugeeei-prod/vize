#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/nuxt-generated-membership/require-real-backends-and-preserve-source-identity.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
test -z "${VIZE_TEST_DISABLE_TSGO:-}"
test -f tests/node_modules/vue-tsc/bin/vue-tsc.js
mkdir -p "$RUNNER_TEMP/nuxt-membership-packets"
git rev-parse HEAD > "$RUNNER_TEMP/nuxt-membership-packets/source-sha.txt"
git rev-parse 'HEAD^{tree}' > "$RUNNER_TEMP/nuxt-membership-packets/source-tree.txt"
git status --porcelain > "$RUNNER_TEMP/nuxt-membership-packets/source-status.txt"
