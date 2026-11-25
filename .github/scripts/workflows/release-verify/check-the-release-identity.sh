#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/release-verify/check-the-release-identity.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
[[ "$RELEASE_VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z-]+(\.[0-9A-Za-z-]+)*)?$ ]]
[[ "$RELEASE_TAG" == "v$RELEASE_VERSION" ]]
[[ "$(git rev-parse HEAD)" == "$RELEASE_SHA" ]]
