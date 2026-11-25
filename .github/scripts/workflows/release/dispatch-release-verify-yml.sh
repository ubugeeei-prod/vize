#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/release/dispatch-release-verify-yml.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
gh workflow run release-verify.yml --repo "$GITHUB_REPOSITORY" --ref main \
  -f version="$RELEASE_VERSION" -f tag="$RELEASE_TAG" -f sha="$RELEASE_SHA" \
  -f packages="$RELEASE_PACKAGES"
