#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/release-verify/wait-for-npm-and-smoke-the-published-cli.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
read -ra packages <<< "$RELEASE_PACKAGES"
node tools/support/release/verify_published/main.ts "$RELEASE_VERSION" "${packages[@]}"
